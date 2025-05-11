#!/usr/bin/env python3
"""
WXS Generator Script for WiX Toolset

This script recursively scans a folder structure and generates a WXS file
that can be used with the WiX Toolset to create an MSI installer.
It also adds a shortcut to the application in the Start Menu.

Usage: python generate_wxs.py path/to/mview6-windows [output.wxs]
"""

import os
import re
import sys
import uuid
from xml.dom import minidom
from xml.etree import ElementTree as ET

EXTENSIONS = [
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
    ("avif", "image/avif"),
    ("heic", "image/heic"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("svgz", "image/svg+xml"),
    ("pdf", "application/pdf"),
    ("epub", "application/epub+zip"),
    # ("tiff", "image/tiff"),
    # ("tif", "image/tiff"),
    # ("ico", "image/x-icon"),
]

# Add XML namespace for WiX
ET.register_namespace("", "http://schemas.microsoft.com/wix/2006/wi")


def sanitize_id(name):
    """Create a valid WiX ID from a file or directory name."""
    # Remove invalid characters and replace spaces/dots with underscores
    name = re.sub(r"[^a-zA-Z0-9_]", "_", name)

    # Ensure ID doesn't start with a number
    if name and name[0].isdigit():
        name = "id_" + name

    # Avoid empty names
    if not name:
        name = "id_" + str(uuid.uuid4()).replace("-", "")

    return name


def create_directory_element(parent_element, dir_name, dir_id):
    """Create a Directory element in the XML."""
    return ET.SubElement(parent_element, "Directory", {"Id": dir_id, "Name": dir_name})


def create_component_element(parent_element, file_id, relative_path, source_path):
    """Create a Component and File element in the XML."""
    component = ET.SubElement(
        parent_element,
        "Component",
        {"Id": f"Comp_{file_id}", "Guid": "*"},
    )

    ET.SubElement(
        component, "File", {"Id": file_id, "Source": source_path, "KeyPath": "yes"}
    )

    return component


def generate_wxs(root_folder, output_file="mview6.wxs"):
    """Generate a WXS file from a folder structure."""

    # Initialize the XML structure
    wix = ET.Element("{http://schemas.microsoft.com/wix/2006/wi}Wix")
    product = ET.SubElement(
        wix,
        "Product",
        {
            "Id": "69c966bc-c892-421f-a9d0-749e21a0745a",
            "Name": "MView6",
            "Language": "1033",
            "Version": "1.0.0.0",
            "Manufacturer": "NewInnovations",
            "UpgradeCode": str(uuid.uuid4()),
        },
    )

    # Add package information
    ET.SubElement(
        product,
        "Package",
        {"InstallerVersion": "200", "Compressed": "yes", "InstallScope": "perMachine"},
    )

    ET.SubElement(product, "MediaTemplate", {"EmbedCab": "yes"})

    # Define the icon
    ET.SubElement(product, "Icon", Id="MView6Icon", SourceFile="resources/mview6.ico")

    # Add a Property to show icon in Add/Remove Programs
    ET.SubElement(product, "Property", Id="ARPPRODUCTICON", Value="MView6Icon")

    # Create feature element
    feature = ET.SubElement(
        product, "Feature", {"Id": "ProductFeature", "Title": "MView6", "Level": "1"}
    )

    # Root directory structure
    directory_root = ET.SubElement(
        product, "Directory", {"Id": "TARGETDIR", "Name": "SourceDir"}
    )

    # Add Program Files folder
    program_files = ET.SubElement(
        directory_root, "Directory", {"Id": "ProgramFilesFolder"}
    )

    # Add Start Menu Programs folder for shortcut
    program_menu_dir = ET.SubElement(
        directory_root, "Directory", {"Id": "ProgramMenuFolder"}
    )

    # Add application shortcut folder in Start Menu
    app_menu_dir = ET.SubElement(
        program_menu_dir,
        "Directory",
        {"Id": "ApplicationProgramsFolder", "Name": "MView6"},
    )

    # Add main installation folder
    install_folder = ET.SubElement(
        program_files, "Directory", {"Id": "INSTALLFOLDER", "Name": "MView6"}
    )

    # Dictionary to keep track of created directories
    dir_elements = {"": install_folder}  # Empty string is the root
    component_refs = []

    # Add shortcut component
    shortcut_component = ET.SubElement(
        app_menu_dir,
        "Component",
        {"Id": "ApplicationShortcut", "Guid": "*"},
    )

    # Reference the shortcut component in the feature
    ET.SubElement(feature, "ComponentRef", {"Id": "ApplicationShortcut"})
    component_refs.append("ApplicationShortcut")

    # This will be set later when we find the MView6.exe file
    exe_component = None

    # Track file IDs to ensure uniqueness
    used_file_ids = set()
    file_counter = 1

    # Process all files in the directory structure
    for dirpath, dirnames, filenames in os.walk(root_folder):
        # Get the relative path from the root folder
        rel_path = os.path.relpath(dirpath, root_folder)
        if rel_path == ".":
            rel_path = ""

        # Ensure the directory structure exists in our XML
        path_parts = rel_path.split(os.path.sep) if rel_path else []
        current_path = ""
        parent_dir_element = install_folder

        for i, part in enumerate(path_parts):
            current_path = os.path.join(current_path, part) if current_path else part

            if current_path not in dir_elements:
                dir_id = sanitize_id(f"Dir_{part}")
                if i > 0 and len(path_parts) > 1:
                    # Avoid ID collisions for directories with the same name
                    dir_id = f"{dir_id}_{i}"

                parent_dir_element = dir_elements.get(
                    os.path.dirname(current_path) if i > 0 else "", install_folder
                )
                dir_element = create_directory_element(parent_dir_element, part, dir_id)
                dir_elements[current_path] = dir_element

            parent_dir_element = dir_elements[current_path]

        # Process files in the current directory
        for filename in filenames:
            # Create a unique ID for each file
            base_id = sanitize_id(os.path.splitext(filename)[0])
            file_id = f"File_{base_id}"

            # Ensure ID is unique
            while file_id in used_file_ids:
                file_id = f"File_{base_id}_{file_counter}"
                file_counter += 1

            used_file_ids.add(file_id)

            # Source path is relative to the script location
            source_path = os.path.join(dirpath, filename)

            # Create component
            component = create_component_element(
                parent_dir_element,
                file_id,
                os.path.join(rel_path, filename) if rel_path else filename,
                source_path,
            )

            # Check if this is MView6.exe in the bin directory
            if filename.lower() == "mview6.exe" and "bin" in rel_path.lower().split(
                os.path.sep
            ):
                exe_component = component

            # Add component reference to the feature
            comp_id = f"Comp_{file_id}"
            ET.SubElement(feature, "ComponentRef", {"Id": comp_id})
            component_refs.append(comp_id)

    # Add the shortcut to MView6.exe if we found it
    if exe_component is not None:
        # Add shortcut to the Start Menu
        ET.SubElement(
            shortcut_component,
            "Shortcut",
            {
                "Id": "ApplicationStartMenuShortcut",
                "Name": "MView6",
                "Description": "Launch MView6 Application",
                "Target": "[INSTALLFOLDER]bin\\MView6.exe",
                "WorkingDirectory": "INSTALLFOLDER",
            },
        )

        # Add registry value to control the shortcut component
        ET.SubElement(
            shortcut_component,
            "RegistryValue",
            {
                "Root": "HKCU",
                "Key": "Software\\MView6",
                "Name": "installed",
                "Type": "integer",
                "Value": "1",
                "KeyPath": "yes",
            },
        )

        # Make sure the start menu directory is removed on uninstall
        ET.SubElement(
            shortcut_component,
            "RemoveFolder",
            {
                "Id": "RemoveApplicationProgramsFolder",
                "Directory": "ApplicationProgramsFolder",
                "On": "uninstall",
            },
        )

        # Group extensions by content type for better organization
        ext_by_type = {}
        for ext, content_type in EXTENSIONS:
            # content_type = content_types.get(ext.lower().lstrip("."), f"application/{ext}")
            if content_type not in ext_by_type:
                ext_by_type[content_type] = []
            ext_by_type[content_type].append(ext.lower().lstrip("."))

        # Include the icon
        ET.SubElement(
            exe_component, "File", Id="File_MView6Icon", Source="resources/mview6.ico"
        )

        # Create ProgId elements grouped by content type
        for content_type, exts in ext_by_type.items():
            # Use first extension as identifier
            first_ext = exts[0]
            prog_id = ET.SubElement(
                exe_component,
                "ProgId",
                Id=f"MView6.{first_ext}file",
                Description="MView6",
                Icon="File_MView6Icon",
            )

            # Add extension elements
            for ext in exts:
                extension = ET.SubElement(
                    prog_id, "Extension", Id=ext, ContentType=content_type
                )

                ET.SubElement(
                    extension,
                    "Verb",
                    Id=f"open_{ext}",
                    Command="Open",
                    TargetFile="File_MView6",
                    Argument='"%1"',
                )

            # Add registry keys to ensure Windows recognizes the file association
            regkey = ET.SubElement(
                exe_component,
                "RegistryKey",
                Root="HKCR",
                Key=f"MView6.{first_ext}file\\shell\\open\\command",
                # Action="createAndRemoveOnUninstall",
            )

            ET.SubElement(
                regkey,
                "RegistryValue",
                Type="string",
                Value='[INSTALLFOLDER]bin\\MView6.exe "%1"',
            )
    else:
        print("Warning: Could not find MView6.exe in the bin directory.")

    # Write the XML to a file with pretty formatting
    ET.ElementTree(wix)
    xml_str = ET.tostring(wix, encoding="utf-8")
    pretty_xml = minidom.parseString(xml_str).toprettyxml(indent="  ")

    # Remove extra blank lines from the output
    pretty_xml = "\n".join([line for line in pretty_xml.split("\n") if line.strip()])

    with open(output_file, "w", encoding="utf-8") as f:
        f.write(pretty_xml)

    return pretty_xml, len(component_refs)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} path/to/mview6-windows [output.wxs]")
        sys.exit(1)

    root_folder = sys.argv[1]

    if not os.path.isdir(root_folder):
        print(f"Error: {root_folder} is not a valid directory")
        sys.exit(1)

    output_file = sys.argv[2] if len(sys.argv) > 2 else "mview6.wxs"

    try:
        _, file_count = generate_wxs(root_folder, output_file)
        print(f"Successfully generated {output_file} with {file_count} files")
    except Exception as e:
        print(f"Error generating WXS file: {e}")
        sys.exit(1)
