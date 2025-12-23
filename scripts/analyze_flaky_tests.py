#!/usr/bin/env python3
import os
import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
import glob
import argparse
import re

def parse_os_from_filename(filepath):
    """Extracts OS name from filename (e.g., junit-implants-ubuntu-latest.xml)."""
    basename = os.path.basename(filepath)
    # Common OS patterns in GitHub Actions
    if 'ubuntu' in basename.lower():
        return 'ubuntu-latest'
    if 'macos' in basename.lower():
        return 'macos-latest'
    if 'windows' in basename.lower():
        return 'windows-latest'
    return 'unknown'

def parse_junit_xml(filepath):
    """Parses a JUnit XML file and returns a list of test results."""
    results = []
    os_name = parse_os_from_filename(filepath)

    try:
        tree = ET.parse(filepath)
        root = tree.getroot()

        # Handle both single testsuite and testsuites root elements
        suites = []
        if root.tag == 'testsuites':
            suites = root.findall('testsuite')
        elif root.tag == 'testsuite':
            suites = [root]
        else:
            # Try to find any testsuite elements
            suites = root.findall('.//testsuite')

        for suite in suites:
            for case in suite.findall('testcase'):
                # Construct a unique identifier for the test
                classname = case.get('classname', 'unknown')
                name = case.get('name', 'unknown')
                test_id = f"{classname}::{name}"

                status = 'pass'
                # Check for failure or error
                if case.find('failure') is not None:
                    status = 'fail'
                elif case.find('error') is not None:
                    status = 'fail'
                elif case.find('skipped') is not None:
                    status = 'skip'

                results.append({
                    'id': test_id,
                    'status': status,
                    'file': filepath,
                    'os': os_name
                })
    except ET.ParseError as e:
        print(f"Error parsing {filepath}: {e}", file=sys.stderr)
    except Exception as e:
        print(f"Unexpected error processing {filepath}: {e}", file=sys.stderr)

    return results

def analyze_results(results):
    """Aggregates results and calculates flakiness."""
    # stats[test_id][os_name] = {total, pass, fail, skip}
    stats = defaultdict(lambda: defaultdict(lambda: {'total': 0, 'pass': 0, 'fail': 0, 'skip': 0}))

    for r in results:
        test_id = r['id']
        os_name = r['os']
        status = r['status']
        stats[test_id][os_name]['total'] += 1
        stats[test_id][os_name][status] += 1
        # Also aggregate to 'all'
        stats[test_id]['all']['total'] += 1
        stats[test_id]['all'][status] += 1

    # Calculate failure rate
    flaky_candidates = []
    for test_id, os_data in stats.items():
        # Check 'all' first to see if there are any failures at all
        if os_data['all']['fail'] > 0:
            # Add entry for each OS that has failures
            for os_name, data in os_data.items():
                if os_name == 'all': continue # Skip summary for now, or include it?
                if data['fail'] > 0:
                    fail_rate = data['fail'] / data['total']
                    flaky_candidates.append({
                        'id': test_id,
                        'os': os_name,
                        'fail_rate': fail_rate,
                        'total': data['total'],
                        'fail': data['fail'],
                        'pass': data['pass']
                    })

    # Sort by failure rate (descending) then by total runs (descending)
    flaky_candidates.sort(key=lambda x: (-x['fail_rate'], -x['total']))

    return flaky_candidates

def generate_markdown_report(flaky_tests, limit=5):
    """Generates a Markdown report of top flaky tests."""
    if not flaky_tests:
        return "## 🛡️ Test Health Report\n\nNo flaky tests detected in this batch! 🎉"

    report = ["## 🛡️ Test Health Report\n"]
    report.append(f"Found {len(flaky_tests)} flaky test entries (grouped by OS).\n")
    report.append("### Top Flaky Offenders\n")
    report.append("| Test Name | OS | Failure Rate | Failures | Total Runs |")
    report.append("|-----------|:--:|:------------:|:--------:|:----------:|")

    for test in flaky_tests[:limit]:
        rate = f"{test['fail_rate']:.1%}"
        report.append(f"| `{test['id']}` | {test['os']} | {rate} | {test['fail']} | {test['total']} |")

    if len(flaky_tests) > limit:
        report.append(f"\n*...and {len(flaky_tests) - limit} more.*")

    return "\n".join(report)

def main():
    parser = argparse.ArgumentParser(description='Analyze JUnit XML files for flaky tests.')
    parser.add_argument('directory', help='Directory containing JUnit XML files')
    parser.add_argument('--output', help='Output markdown file path', default=None)
    args = parser.parse_args()

    if not os.path.isdir(args.directory):
        print(f"Directory not found: {args.directory}")
        sys.exit(1)

    xml_files = glob.glob(os.path.join(args.directory, '**/*.xml'), recursive=True)
    if not xml_files:
        print(f"No XML files found in {args.directory}")
        # Not an error, just no results
        sys.exit(0)

    all_results = []
    for xml_file in xml_files:
        all_results.extend(parse_junit_xml(xml_file))

    flaky_stats = analyze_results(all_results)
    markdown_report = generate_markdown_report(flaky_stats)

    print(markdown_report)

    if args.output:
        with open(args.output, 'w') as f:
            f.write(markdown_report)

if __name__ == '__main__':
    main()
