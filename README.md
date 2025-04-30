# cnreader
A desktop app that helps reading Chinese texts. you can:
- select a word and see its meaning
- OCR files and clipboard images
- translate selected text through Deepl
- ask Chat GPT / Deepseek about the meaning or usage examples with just one click
- listen to pronounciation
- convert traditional to simplified
- look for a word in your local Anki database

To compile it use: cargo build --release

Requirements: SQLite3 library, Wayland on Linux

For OCR models look at: https://huggingface.co/SWHL/RapidOCR/tree/main/PP-OCRv4
Download ch_PP-OCRv4_det_infer.onnx, ch_PP-OCRv4_rec_infer.onnx and ppocr_keys_v1.txt and point to their directory in the app.toml file.
