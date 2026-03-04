import argparse
import os
from pathlib import Path
from typing import List, Dict, Union
import jinja2

# Define a type alias for our version dictionary for better readability
VersionInfo = Dict[str, Union[str, bool]]


def get_version_metadata(version_id: str) -> VersionInfo:
    """
    Helper to provide 'nice' names and tags based on folder names.
    """
    is_latest: bool = False
    tag: str = "Stable"
    name: str = version_id

    if version_id in "main":
        tag = "Latest"
        name = "Main Branch"
        is_latest = True

    return {
        "id": version_id,
        "name": name,
        "tag": tag,
        "is_latest": is_latest,
        "description": f"Documentation for {name}.",
    }


def main() -> None:
    parser: argparse.ArgumentParser = argparse.ArgumentParser(
        description="Generate a landing page for mdBook versions."
    )
    parser.add_argument(
        "--output",
        required=True,
        help="The directory where versions exist and where index.html will be saved.",
    )
    args: argparse.Namespace = parser.parse_args()

    output_path: Path = Path(args.output)
    script_dir: Path = Path(__file__).parent.resolve()

    if not output_path.exists():
        print(f"Error: Output directory '{output_path}' does not exist.")
        return

    # 1. Scan the output directory for subdirectories (versions)
    version_ids: List[str] = [
        d
        for d in os.listdir(output_path)
        if os.path.isdir(output_path / d) and not d.startswith(".")
    ]

    # Sort versions (reverse alphabetical usually puts higher versions/main at top)
    version_ids.sort(reverse=False)

    # 2. Build the version data list
    versions: List[VersionInfo] = [get_version_metadata(vid) for vid in version_ids]

    # 3. Setup Jinja2
    # Use script_dir for the template loader
    template_loader: jinja2.FileSystemLoader = jinja2.FileSystemLoader(
        searchpath=str(script_dir)
    )
    template_env: jinja2.Environment = jinja2.Environment(loader=template_loader)

    try:
        template: jinja2.Template = template_env.get_template("index.html.j2")
    except jinja2.TemplateNotFound:
        print(f"Error: 'index.html.j2' template not found in {script_dir}")
        return

    # 4. Render and save to the output directory
    rendered_html: str = template.render(versions=versions)

    index_file: Path = output_path / "index.html"
    with open(index_file, "w", encoding="utf-8") as f:
        f.write(rendered_html)

    print(f"Successfully generated {index_file} with {len(versions)} versions.")


if __name__ == "__main__":
    main()
