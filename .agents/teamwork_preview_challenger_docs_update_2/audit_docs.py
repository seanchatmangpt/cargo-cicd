import os
import re

files_to_check = [
    "/Users/sac/cargo-cicd/README.md",
    "/Users/sac/cargo-cicd/docs/INDEX.md",
    "/Users/sac/cargo-cicd/docs/star-toml-refactor/PRD.md",
    "/Users/sac/cargo-cicd/docs/star-toml-refactor/ARD.md",
    "/Users/sac/cargo-cicd/docs/star-toml-refactor/REFACTOR.md"
]

forbidden_terms = [
    "ALIVE",
    "Inspection Gate",
    "wall",
    "Nehemiah",
    "Field8",
    "Instinct8",
    "Cargo Court",
    "AGI",
    "Truex",
    "CONSTRUCT8"
]

# We will check both exact match (case-sensitive) and case-insensitive.
# "wall" is checked as whole-word only.

def run_audit():
    results = []
    
    for file_path in files_to_check:
        if not os.path.exists(file_path):
            print(f"WARNING: File not found: {file_path}")
            results.append({
                "file": file_path,
                "status": "NOT_FOUND",
                "matches": []
            })
            continue
            
        print(f"Auditing {file_path}...")
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
            
        file_matches = []
        for line_num, line in enumerate(lines, 1):
            # Check for each term
            for term in forbidden_terms:
                if term == "wall":
                    # Whole word check
                    # Case-sensitive
                    pattern_cs = r'\bwall\b'
                    # Case-insensitive
                    pattern_ci = r'\bwall\b'
                    
                    matches_cs = re.findall(pattern_cs, line)
                    matches_ci = re.findall(pattern_ci, line, re.IGNORECASE)
                    
                    if matches_cs:
                        file_matches.append({
                            "line_num": line_num,
                            "term": "wall (exact)",
                            "line_content": line.strip()
                        })
                    elif matches_ci:
                        file_matches.append({
                            "line_num": line_num,
                            "term": "wall (case-insensitive)",
                            "line_content": line.strip()
                        })
                else:
                    # Generic terms
                    # Case-sensitive check
                    if term in line:
                        file_matches.append({
                            "line_num": line_num,
                            "term": f"{term} (exact)",
                            "line_content": line.strip()
                        })
                    # Case-insensitive check (if not matched exactly)
                    elif term.lower() in line.lower():
                        file_matches.append({
                            "line_num": line_num,
                            "term": f"{term} (case-insensitive)",
                            "line_content": line.strip()
                        })
                        
        results.append({
            "file": file_path,
            "status": "OK",
            "matches": file_matches
        })
        
    # Print report
    print("\n--- AUDIT REPORT ---")
    total_matches = 0
    for res in results:
        print(f"\nFile: {res['file']} (Status: {res['status']})")
        if res['matches']:
            print(f"Found {len(res['matches'])} occurrences:")
            for m in res['matches']:
                print(f"  Line {m['line_num']}: [{m['term']}] -> {m['line_content']}")
                total_matches += 1
        else:
            print("  No forbidden terms found.")
            
    print(f"\nTotal findings: {total_matches}")

if __name__ == "__main__":
    run_audit()
