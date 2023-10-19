import argparse
import json


def generate_summary_markdown(
    base_covdir, incoming_covdir, base_branch, incoming_branch
):
    """Generate a markdown summary with the coverage differences."""

    root_base_coverage = extract_root_coverage(base_covdir)
    root_incoming_coverage = extract_root_coverage(incoming_covdir)

    coverage_diff = (
        root_incoming_coverage["coverage_percent"]
        - root_base_coverage["coverage_percent"]
    )
    total_files_diff = (
        root_incoming_coverage["total_files"] - root_base_coverage["total_files"]
    )
    total_lines_diff = (
        root_incoming_coverage["total_lines"] - root_base_coverage["total_lines"]
    )
    covered_lines_diff = (
        root_incoming_coverage["covered_lines"] - root_base_coverage["covered_lines"]
    )
    missed_lines_diff = (
        root_incoming_coverage["missed_lines"] - root_base_coverage["missed_lines"]
    )

    get_comparison_symbol = lambda x: "+" if x > 0 else "-" if x < 0 else " "

    # Generate table data
    subtitle = ["##", base_branch, incoming_branch, "+/-", "##"]
    coverage = [
        f"{get_comparison_symbol(coverage_diff)} Coverage",
        f"{root_base_coverage['coverage_percent']:.2f}%",
        f"{root_incoming_coverage['coverage_percent']:.2f}%",
        f"{'+' if coverage_diff > 0 else ''}{coverage_diff:.2f}%",
        "",
    ]
    files = [
        f"{get_comparison_symbol(total_files_diff)} Files",
        root_base_coverage["total_files"],
        root_incoming_coverage["total_files"],
        f"{'+' if total_files_diff > 0 else ''}{total_files_diff or ''}",
        "",
    ]
    lines = [
        f"{get_comparison_symbol(total_lines_diff)} Lines",
        root_base_coverage["total_lines"],
        root_incoming_coverage["total_lines"],
        f"{'+' if total_lines_diff > 0 else ''}{total_lines_diff or ''}",
        "",
    ]
    hits = [
        f"{get_comparison_symbol(covered_lines_diff)} Hits",
        root_base_coverage["covered_lines"],
        root_incoming_coverage["covered_lines"],
        f"{'+' if covered_lines_diff > 0 else ''}{covered_lines_diff or ''}",
        "",
    ]
    misses = [
        f"{get_comparison_symbol(missed_lines_diff)} Misses",
        root_base_coverage["missed_lines"],
        root_incoming_coverage["missed_lines"],
        f"{'+' if missed_lines_diff > 0 else ''}{missed_lines_diff or ''}",
        "",
    ]
    rows = [subtitle, coverage, files, lines, hits, misses]

    # Format table
    get_widest_column = lambda x: max([len(str(row[x])) for row in rows])
    padding = 3
    for i in range(len(rows[0])):
        widest_column = get_widest_column(i)
        for row in rows:
            if i == 0:
                row[i] = str(row[i]).ljust(widest_column)
            else:
                row[i] = str(row[i]).rjust(widest_column + padding)

    # Generate table string
    table_rows = ["".join(row) for row in rows]
    table_width = len(table_rows[0])
    separator = "=" * table_width
    table_rows.insert(0, f"@@{'Coverage Diff'.center(table_width - 4)}@@")
    table_rows.insert(2, separator)
    table_rows.insert(6, separator)
    table = "\n".join(table_rows)

    return table


def generate_comparison_markdown(base_covdir, incoming_covdir, base_html_url):
    """Generate a markdown table with the coverage differences."""
    differences = compare_covdir_files(base_covdir, incoming_covdir)

    markdown_table_data = []

    # Table headers
    markdown_table_data.append(["Files Changed", "Coverage", ""])
    markdown_table_data.append(["---", "---", "---"])

    for item in differences:
        # Determine emoji based on coverage difference
        if item["diff"] > 0:
            emoji = "🔼"
        elif item["diff"] < 0:
            emoji = "🔽"
        else:
            emoji = ""

        # Append row data
        row = [
            f"[{item['path']}]({base_html_url}/html{item['path']}.html)",
            f"{item['incoming_coverage']:.2f}% ({'+' if item['diff'] > 0 else ''}{item['diff']:.2f}%)",
            emoji,
        ]
        markdown_table_data.append(row)

    # Convert table data to markdown format
    markdown_table = "\n".join(
        ["| " + " | ".join(row) + " |" for row in markdown_table_data]
    )

    return markdown_table


def compare_covdir_files(base_covdir, incoming_covdir):
    """Compare two covdir data sets for files only and return differences."""
    base_covdir_data = extract_files_coverage(base_covdir)
    incoming_covdir_data = extract_files_coverage(incoming_covdir)

    # Convert to dictionary for easier lookup
    base_covdir_dict = {item["path"]: item["coverage"] for item in base_covdir_data}
    incoming_covdir_dict = {
        item["path"]: item["coverage"] for item in incoming_covdir_data
    }

    differences = []
    for path, incoming_coverage in incoming_covdir_dict.items():
        if path in base_covdir_dict:
            base_coverage = base_covdir_dict[path]
            diff = incoming_coverage - base_coverage
            if diff != 0:
                differences.append(
                    {
                        "path": path,
                        "base_coverage": base_coverage,
                        "incoming_coverage": incoming_coverage,
                        "diff": diff,
                    }
                )

    return differences


def extract_root_coverage(data):
    """Get the coverage summary of the root node."""

    total_files = len(extract_files_coverage(data))

    return {
        "coverage_percent": data["coveragePercent"],
        "total_lines": data["linesTotal"],
        "covered_lines": data["linesCovered"],
        "missed_lines": data["linesMissed"],
        "total_files": total_files,
    }


def extract_files_coverage(data, path=""):
    """Recursively extract coverage data from the nested structure, considering only leaf nodes."""
    results = []

    # If the node has children, explore the children nodes
    if "children" in data:
        for key in data["children"]:
            results.extend(
                extract_files_coverage(data["children"][key], path + data["name"] + "/")
            )
    # If the node doesn't have children, consider it a leaf node (file) and extract its coverage
    elif "name" in data and "coveragePercent" in data:
        results.append(
            {"path": path + data["name"], "coverage": data["coveragePercent"]}
        )
    return results


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        prog="coverage-report",
        description="This script compares coverage between two covdirs and generates a report markdown summary and table with the differences.",
    )
    parser.add_argument(
        "--base-covdir",
        metavar="path",
        required=True,
        help="covdir path for the base branch",
    )
    parser.add_argument(
        "--incoming-covdir",
        metavar="path",
        required=True,
        help="covdir path for the incoming branch",
    )
    parser.add_argument(
        "--base-branch", metavar="branch", required=True, help="name of the base branch"
    )
    parser.add_argument(
        "--incoming-branch",
        metavar="branch",
        required=True,
        help="name of the incoming branch",
    )
    parser.add_argument(
        "--base-html-url",
        metavar="url",
        required=True,
        help="URL to the base HTML coverage report",
    )
    parser.add_argument(
        "--output",
        metavar="path",
        required=False,
        help="path to the output file, if not specified, the output will be printed to stdout",
    )
    args = parser.parse_args()

    # Open covdir files
    with open(args.base_covdir, "r") as f:
        base_covdir = json.load(f)
    with open(args.incoming_covdir, "r") as f:
        incoming_covdir = json.load(f)

    # Generate markdown summary
    markdown_summary = generate_summary_markdown(
        base_covdir,
        incoming_covdir,
        args.base_branch,
        args.incoming_branch,
    )

    # Generate markdown table
    markdown_table = generate_comparison_markdown(
        base_covdir, incoming_covdir, args.base_html_url
    )

    # Generate output
    output = f"""```diff
{markdown_summary}
```

{markdown_table}
"""

    if args.output:
        with open(args.output, "w") as f:
            f.write(output)
    else:
        print(output)
