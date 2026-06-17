#!/usr/bin/env python3
"""Convert thesis markdown to PDF via HTML."""
import markdown
import subprocess
import sys

with open("/home/user/cargo-cicd/thesis_full.md", "r") as f:
    md_content = f.read()

html_body = markdown.markdown(
    md_content,
    extensions=["tables", "fenced_code", "toc"]
)

html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>cargo-cicd: A Level-5 Process-Data Engine for Local-First CI/CD Orchestration in Rust Workspaces</title>
<style>
  @page {{ margin: 2.5cm; }}
  body {{
    font-family: "Times New Roman", Times, serif;
    font-size: 12pt;
    line-height: 1.6;
    color: #000;
    max-width: 800px;
    margin: 0 auto;
    padding: 2em;
  }}
  h1 {{ font-size: 18pt; margin-top: 2em; page-break-before: always; }}
  h1:first-child {{ page-break-before: avoid; }}
  h2 {{ font-size: 14pt; margin-top: 1.5em; }}
  h3 {{ font-size: 12pt; margin-top: 1em; }}
  pre, code {{
    font-family: "Courier New", monospace;
    font-size: 10pt;
    background: #f5f5f5;
    padding: 0.2em 0.4em;
  }}
  pre {{ padding: 1em; overflow-x: auto; }}
  table {{
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
    font-size: 10pt;
  }}
  th, td {{
    border: 1px solid #ccc;
    padding: 0.4em 0.8em;
    text-align: left;
  }}
  th {{ background: #eee; font-weight: bold; }}
  hr {{ border: none; border-top: 1px solid #999; margin: 2em 0; }}
  blockquote {{ border-left: 3px solid #999; padding-left: 1em; color: #555; }}
  .toc {{ background: #f9f9f9; padding: 1em; border: 1px solid #ddd; margin-bottom: 2em; }}
</style>
</head>
<body>
<div style="text-align:center; margin-bottom:3em; padding-top:3em;">
  <h1 style="font-size:22pt; page-break-before:avoid; border-bottom:2px solid #000; padding-bottom:0.5em;">
    cargo-cicd: A Level-5 Process-Data Engine<br>
    for Local-First CI/CD Orchestration<br>
    in Rust Workspaces
  </h1>
  <p style="font-size:14pt;">PhD Thesis</p>
  <p style="font-size:12pt; color:#555;">Department of Computer Science<br>
  School of Software Engineering<br>
  Version 26.6.2 &mdash; June 2026</p>
</div>
{html_body}
</body>
</html>"""

with open("/home/user/cargo-cicd/thesis.html", "w") as f:
    f.write(html)

print("Generated thesis.html")

# Try to convert to PDF using available tools
tools = [
    ["wkhtmltopdf", "--page-size", "A4", "--margin-top", "25mm",
     "--margin-bottom", "25mm", "--margin-left", "25mm", "--margin-right", "25mm",
     "/home/user/cargo-cicd/thesis.html", "/home/user/cargo-cicd/thesis.pdf"],
    ["weasyprint", "/home/user/cargo-cicd/thesis.html", "/home/user/cargo-cicd/thesis.pdf"],
    ["chromium", "--headless", "--no-sandbox", "--disable-gpu",
     "--print-to-pdf=/home/user/cargo-cicd/thesis.pdf",
     "/home/user/cargo-cicd/thesis.html"],
    ["chromium-browser", "--headless", "--no-sandbox", "--disable-gpu",
     "--print-to-pdf=/home/user/cargo-cicd/thesis.pdf",
     "/home/user/cargo-cicd/thesis.html"],
    ["google-chrome", "--headless", "--no-sandbox", "--disable-gpu",
     "--print-to-pdf=/home/user/cargo-cicd/thesis.pdf",
     "/home/user/cargo-cicd/thesis.html"],
]

for tool_cmd in tools:
    try:
        result = subprocess.run(tool_cmd, capture_output=True, timeout=60)
        if result.returncode == 0:
            print(f"Generated thesis.pdf using {tool_cmd[0]}")
            sys.exit(0)
    except (FileNotFoundError, subprocess.TimeoutExpired):
        continue

print("PDF generation tools not available. HTML thesis written to thesis.html")
print("To convert: pandoc thesis_full.md -o thesis.pdf  OR  wkhtmltopdf thesis.html thesis.pdf")
