Name:       	xdman_gtk
Version:    	8.0.25
Release:    	1%{?dist}
Summary:    	Xtreme Download Manager
AutoReqProv: 	no
Requires:   	gtk3 >= 3.22
Requires:   	(ffmpeg-free or ffmpeg)

Group:      	System Environment/Base
License:    	GPLv3+
Source0:    	xdman_gtk-8.0.25.tar.gz

%description
Open source download accelerator and video downloader.

%prep
%setup -q

%install
cp -rfa * %{buildroot}

%files
/usr/bin/*
/usr/share/applications/*
/opt/xdman/*
