import urllib.request
import os

def load_english_translation():
    url = "https://raw.githubusercontent.com/subhra74/xdm/wpf/app/XDM/Lang/English.txt"
    try:
        with urllib.request.urlopen(url) as response:
            text = response.read().decode('utf-8')
            lines = text.splitlines()
            mappings = {}
            for line in lines:
                if '=' in line:
                    key, english_text = line.strip().split('=', 1)
                    mappings[key] = {"englishText": english_text, "text": ""}
            return mappings
    except Exception as e:
        print(f"Error loading English translation: {e}")
        return None

def generate_translation(mappings, language):
    if not language:
        print("Invalid file name.")
        return

    output = []
    for key, data in mappings.items():
        output.append(f"{key}={data['text']}")

    file_content = "\n".join(output)
    file_name = f"{language}.txt"

    with open(file_name, "w", encoding="utf-8") as f:
        f.write(file_content)

    print(f"Translation saved to {file_name}")

def main():
    print("XDM Translation Generator (Python)")
    mappings = load_english_translation()
    if not mappings:
        return

    language = input("Enter target language name (e.g., Spanish): ").strip()
    if not language:
        print("Language name is required.")
        return

    print("\nEnter translations for the following keys (press Enter to skip):")
    for key, data in mappings.items():
        translated = input(f"[{key}] {data['englishText']}: ").strip()
        if translated:
            data['text'] = translated
        else:
            data['text'] = "" # Default empty if not translated

    generate_translation(mappings, language)

if __name__ == "__main__":
    main()
