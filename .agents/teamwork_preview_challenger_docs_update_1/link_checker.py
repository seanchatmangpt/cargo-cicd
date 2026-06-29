import os
import re
import sys

def parse_markdown_links(filepath):
    """
    Parses all markdown links from a file.
    Returns a list of tuples: (line_num, link_text, link_target)
    """
    links = []
    # Match markdown links: [text](target)
    # Be careful with nested brackets or quotes
    pattern = re.compile(r'\[([^\]]*?)\]\(([^)]+?)\)')
    
    with open(filepath, 'r', encoding='utf-8') as f:
        for idx, line in enumerate(f, 1):
            matches = pattern.findall(line)
            for text, target in matches:
                # Strip potential titles in quotes
                target = target.strip()
                # e.g., target could be: path/to/file "title"
                # let's split by whitespace and check if the second part is quoted
                parts = target.split(None, 1)
                if len(parts) > 1 and (parts[1].startswith('"') or parts[1].startswith("'")):
                    target = parts[0]
                links.append((idx, text, target))
    return links

def verify_links(base_dir, file_rel_path, links):
    """
    Verifies target existence of links relative to the referencing file.
    """
    referencing_file = os.path.join(base_dir, file_rel_path)
    referencing_dir = os.path.dirname(referencing_file)
    
    results = []
    for line_num, text, target in links:
        # Ignore external links
        if target.startswith(('http://', 'https://', 'mailto:', 'ftp://')):
            results.append({
                'line': line_num,
                'text': text,
                'target': target,
                'type': 'external',
                'exists': True,
                'resolved_path': target
            })
            continue
        
        # Ignore empty or just hash (anchor within same file)
        if not target or target.startswith('#'):
            results.append({
                'line': line_num,
                'text': text,
                'target': target,
                'type': 'anchor',
                'exists': True,
                'resolved_path': referencing_file
            })
            continue
            
        # Strip fragment identifier if present
        clean_target = target.split('#')[0]
        if not clean_target:
            # e.g., target was just "#anchor" (already handled, but safety check)
            results.append({
                'line': line_num,
                'text': text,
                'target': target,
                'type': 'anchor',
                'exists': True,
                'resolved_path': referencing_file
            })
            continue
            
        # Resolve target path
        if clean_target.startswith('/'):
            # Relative to repository root
            resolved_path = os.path.normpath(os.path.join(base_dir, clean_target.lstrip('/')))
        else:
            # Relative to referencing directory
            resolved_path = os.path.normpath(os.path.join(referencing_dir, clean_target))
            
        exists = os.path.exists(resolved_path)
        results.append({
            'line': line_num,
            'text': text,
            'target': target,
            'type': 'relative',
            'exists': exists,
            'resolved_path': resolved_path
        })
        
    return results

def main():
    repo_root = "/Users/sac/cargo-cicd"
    files_to_check = [
        "README.md",
        "docs/INDEX.md"
    ]
    
    all_ok = True
    dead_links = []
    
    for file_rel in files_to_check:
        full_path = os.path.join(repo_root, file_rel)
        if not os.path.exists(full_path):
            print(f"Error: Referencing file {full_path} does not exist!")
            all_ok = False
            continue
            
        print(f"Checking links in {file_rel}...")
        links = parse_markdown_links(full_path)
        verification_results = verify_links(repo_root, file_rel, links)
        
        file_dead = []
        for res in verification_results:
            if not res['exists']:
                file_dead.append(res)
                all_ok = False
                
        print(f"  Total links found: {len(verification_results)}")
        print(f"  Relative links: {len([r for r in verification_results if r['type'] == 'relative'])}")
        print(f"  Dead links: {len(file_dead)}")
        for dl in file_dead:
            print(f"    Line {dl['line']}: [{dl['text']}]({dl['target']}) -> unresolved path: {dl['resolved_path']}")
            dead_links.append((file_rel, dl))
            
    print("\n--- Summary ---")
    if all_ok:
        print("All relative links verified successfully!")
    else:
        print(f"Found {len(dead_links)} dead links.")
        
if __name__ == "__main__":
    main()
