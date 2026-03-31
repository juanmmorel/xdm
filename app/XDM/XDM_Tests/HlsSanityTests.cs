using System;
using System.IO;
using System.Text;
using System.Threading.Tasks;
using NUnit.Framework;
using XDM.Core;
using System.Threading;
using XDM.Core.MediaParser.Hls;
using Serilog;
using XDM.Core.MediaProcessor;
using System.Net;
using XDM.Core.Downloader.Adaptive.Hls;

namespace XDM.SystemTests
{
    public class HlsSanityTests
    {
        MockServer.MockServer mockServer;
        [SetUp]
        public void Setup()
        {
            ServicePointManager.DefaultConnectionLimit = 100;
            Config.DataDir = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());
            Config.LoadConfig();
            Directory.CreateDirectory(Config.DataDir);

            Log.Logger = new LoggerConfiguration()
                .MinimumLevel.Debug()
                .WriteTo.Console()
                .CreateLogger();
        }

        [TearDown]
        public void TearDown()
        {
            mockServer?.Stop();
        }

        [TestCase(
@"
        #EXTM3U
        #EXT-X-VERSION:3
        #EXT-X-TARGETDURATION:64
        #EXT-X-MEDIA-SEQUENCE:4
        #EXTINF:57.590867,
        hls4.ts
        #EXTINF:62.929533,
        hls5.ts
        #EXTINF:58.391667,
        hls6.ts
        #EXTINF:64.497767,
        hls7.ts
        #EXTINF:31.664967,
        hls8.ts
        #EXT-X-ENDLIST

")]
        public void ParseMediaSegmentsSuccess(string data)
        {
            var pl = HlsParser.ParseMediaSegments(data.Split('\n'),
                    "http://example/hls/playlist.m3u8");
            Assert.NotNull(pl);
            Assert.IsTrue(pl.TotalDuration > 0);
            Assert.IsTrue(pl.MediaSegments.Count > 0);
        }

        [TestCase(".ts", ".ts")]
        [TestCase(".fmp4", ".mp4")]
        [TestCase(".m4s", ".mp4")]
        [TestCase(".mks", ".mkv")]
        public async Task DownloadAsyncWithMockSuccess(string segmentedExt, string expectedExt)
        {
            var n = 20;
            var random = new Random();
            mockServer = new MockServer.MockServer();
            var size = 0L;
            var mockIds = new string[n];
            var streams = new StringBuilder();
            for (var i = 0; i < n; i++)
            {
                var mockId = Guid.NewGuid().ToString() + segmentedExt;
                size += mockServer.AddMockHandler(mockId, start: 1, end: random.Next(1, 4)).Size;
                mockIds[i] = mockId;
                streams.Append(
                    $@"#EXTINF:57.590867
                    {mockServer.BaseUrl}{mockId}
                    ");
            }

            var playlist =
                $@"
                #EXTM3U
                #EXT-X-VERSION:3
                #EXT-X-TARGETDURATION:64
                {streams}
                #EXT-X-ENDLIST
                ";
            var pid = Guid.NewGuid().ToString();
            mockServer.AddMockHandler(pid, contents: Encoding.UTF8.GetBytes(playlist));
            mockServer.StartAsync();

            var url = $"{mockServer.BaseUrl}{pid}";

            string id = Guid.NewGuid().ToString();

            string tempDir = Path.Combine(Path.GetTempPath(), id);
            Directory.CreateDirectory(tempDir);

            Console.WriteLine(tempDir);

            var success = false;
            var cs = new CancellationTokenSource();

            var hc = new MultiSourceHLSDownloader(new MultiSourceHLSDownloadInfo
            {
                VideoUri = url
            },
            mediaProcessor: new FakeMediaProcessor());

            hc.Finished += (a, b) =>
            {
                Console.Write("Success");
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                Console.Write("Failed");
                success = false;
                cs.Cancel();
            };
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName("Sample", hc.FileNameFetchMode);
            hc.Start();
            try
            {
                await Task.Delay(2 * 60 * 1000, cs.Token);
            }
            catch { }

            Assert.IsTrue(success);

            long size2 = hc.FileSize;

            Console.WriteLine(size2 + " " + size);

            Console.WriteLine(hc.Duration);
            Assert.NotZero(hc.Duration);
            Assert.AreEqual(size2, size);
            Assert.IsTrue(expectedExt.Equals(Path.GetExtension(hc.TargetFileName),
                StringComparison.InvariantCultureIgnoreCase));
        }

        private (string Url, long Size) CreateMockPlaylist(int n, MockServer.MockServer mockServer, string ext = "")
        {
            var random = new Random();
            var size = 0L;
            var mockIds = new string[n];
            var streams = new StringBuilder();
            for (var i = 0; i < n; i++)
            {
                var mockId = Guid.NewGuid().ToString() + ext;
                size += mockServer.AddMockHandler(mockId, start: 5, end: random.Next(7, 12)).Size;
                mockIds[i] = mockId;
                streams.Append(
                $@"#EXTINF:57.590867,
                                {mockServer.BaseUrl}{mockId}
                ");
            }

            var playlist = $@"
                                #EXTM3U
                                #EXT-X-VERSION:3
                                #EXT-X-TARGETDURATION:64
                                {streams}
                                #EXT-X-ENDLIST
                                ";

            var pid = Guid.NewGuid().ToString();
            mockServer.AddMockHandler(pid, contents: Encoding.UTF8.GetBytes(playlist));

            var url = $"{mockServer.BaseUrl}{pid}";
            return (url, size);
        }

        [Test]
        public async Task DownloadAsyncWithMockPauseResumeSuccess()
        {
            var success = false;
            var cs = new CancellationTokenSource();

            var n = 10;
            mockServer = new MockServer.MockServer();
            var pl = CreateMockPlaylist(n, mockServer);
            mockServer.StartAsync();

            string id = Guid.NewGuid().ToString();

            string tempDir = Path.Combine(Path.GetTempPath(), id);
            Directory.CreateDirectory(tempDir);

            Console.WriteLine(tempDir);

            var hc = new MultiSourceHLSDownloader(new MultiSourceHLSDownloadInfo
            {
                VideoUri = pl.Url
            },
            mediaProcessor: new FakeMediaProcessor());

            hc.Finished += (a, b) =>
            {
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                success = false;
                cs.Cancel();
            };
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName("Sample", hc.FileNameFetchMode);
            hc.Start();

            await Task.Delay(2000);
            hc.Stop();
            Log.Information("Stopped --");
            await Task.Delay(2000);
            var name = hc.TargetFileName;

            cs = new CancellationTokenSource();
            hc = new MultiSourceHLSDownloader(hc.Id,
            mediaProcessor: new FakeMediaProcessor());
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName(name, hc.FileNameFetchMode);
            hc.Finished += (a, b) =>
            {
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                success = false;
                cs.Cancel();
            };
            hc.Resume();

            try
            {
                await Task.Delay(Int32.MaxValue, cs.Token);
            }
            catch { }

            Assert.IsTrue(success);

            long size2 = hc.FileSize;

            Console.WriteLine(size2 + " " + pl.Size);

            Console.WriteLine(hc.Duration);
            Assert.NotZero(hc.Duration);
            Assert.AreEqual(size2, pl.Size);
        }

        [TestCase(".ts", ".ts")]
        [TestCase(".fmp4", ".mp4")]
        [TestCase(".m4s", ".mp4")]
        [TestCase(".mks", ".mkv")]
        public async Task DownloadAsync_With2Url_Success(string segmentedExt, string expectedExt)
        {
            var success = false;
            var cs = new CancellationTokenSource();

            var n = 10;
            mockServer = new MockServer.MockServer();
            var pl1 = CreateMockPlaylist(n, mockServer, segmentedExt);
            var pl2 = CreateMockPlaylist(n, mockServer, segmentedExt);

            mockServer.StartAsync();

            string id = Guid.NewGuid().ToString();

            string tempDir = Path.Combine(Path.GetTempPath(), id);
            Directory.CreateDirectory(tempDir);

            Console.WriteLine(tempDir);

            var hc = new MultiSourceHLSDownloader(
                new MultiSourceHLSDownloadInfo
                {
                    VideoUri = pl1.Url,
                    AudioUri = pl2.Url
                },
            mediaProcessor: new FakeMediaProcessor());
            hc.Finished += (a, b) =>
            {
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                success = false;
                cs.Cancel();
            };
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName("Sample", hc.FileNameFetchMode);
            hc.Start();
            try
            {
                await Task.Delay(5 * 60 * 1000, cs.Token);
            }
            catch { }

            Assert.IsTrue(success);
            Console.WriteLine(hc.Duration);
            Assert.NotZero(hc.Duration);

            long size2 = hc.FileSize;

            Assert.AreEqual(pl1.Size + pl2.Size, size2);
            Assert.IsTrue(expectedExt.Equals(Path.GetExtension(hc.TargetFileName),
                StringComparison.InvariantCultureIgnoreCase));
        }

        [TestCase(".ts", ".ts")]
        [TestCase(".fmp4", ".mp4")]
        [TestCase(".m4s", ".mp4")]
        [TestCase(".mks", ".mkv")]
        public async Task DownloadAsync_With2Url_PauseResume_Success(string segmentedExt, string expectedExt)
        {
            var success = false;
            var cs = new CancellationTokenSource();

            var n = 20;
            mockServer = new MockServer.MockServer();
            var pl1 = CreateMockPlaylist(n, mockServer, segmentedExt);
            var pl2 = CreateMockPlaylist(n, mockServer, segmentedExt);

            mockServer.StartAsync();

            string id = Guid.NewGuid().ToString();

            string tempDir = Path.Combine(Path.GetTempPath(), id);
            Directory.CreateDirectory(tempDir);

            var hc = new MultiSourceHLSDownloader(
                new MultiSourceHLSDownloadInfo
                {
                    VideoUri = pl1.Url,
                    AudioUri = pl2.Url
                },
            mediaProcessor: new FakeMediaProcessor());

            hc.Finished += (a, b) =>
            {
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                success = false;
                cs.Cancel();
            };
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName("Sample", hc.FileNameFetchMode);
            hc.Start();

            await Task.Delay(10000);
            hc.Stop();
            Log.Information("Stopped --");
            await Task.Delay(2000);
            var name = hc.TargetFileName;

            cs = new CancellationTokenSource();
            hc = new MultiSourceHLSDownloader(hc.Id,
            mediaProcessor: new FakeMediaProcessor());
            hc.SetTargetDirectory(tempDir);
            hc.SetFileName(name, hc.FileNameFetchMode);
            hc.Finished += (a, b) =>
            {
                Log.Debug("Finished2");
                success = true;
                cs.Cancel();
            };
            hc.Failed += (a, b) =>
            {
                success = false;
                cs.Cancel();
            };
            hc.Resume();

            try
            {
                await Task.Delay(15 * 60 * 1000, cs.Token);
            }
            catch { }

            Assert.IsTrue(success);

            long size2 = hc.FileSize;
            Log.Debug(size2 + " " + (pl1.Size + pl2.Size));
            Assert.AreEqual(pl1.Size + pl2.Size, size2);

            Log.Debug(expectedExt + " " + Path.GetExtension(hc.TargetFileName));

            Assert.IsTrue(expectedExt.Equals(Path.GetExtension(hc.TargetFileName),
                StringComparison.InvariantCultureIgnoreCase));
        }
    }

    class FakeMediaProcessor : BaseMediaProcessor
    {
        public override MediaProcessingResult MergeAudioVideStream(string file1, string file2, string outfile, CancelFlag cancellationToken, out long outFileSize)
        {
            Log.Information(file1 + " " + file2 + " " + outfile);
            outFileSize = new FileInfo(file1).Length + new FileInfo(file2).Length;
            return MediaProcessingResult.Success;
        }

        public override MediaProcessingResult MergeHLSAudioVideStream(string segmentListFile, string outfile, CancelFlag cancellationToken, out long outFileSize)
        {
            Log.Information(segmentListFile + " " + outfile);
            outFileSize = -1;
            return MediaProcessingResult.Success;
        }

        public override MediaProcessingResult ConvertToMp3Audio(string segmentListFile, string outfile, CancelFlag cancellationToken, out long outFileSize)
        {
            Log.Information(segmentListFile + " " + outfile);
            outFileSize = -1;
            return MediaProcessingResult.Success;
        }
    }
}
