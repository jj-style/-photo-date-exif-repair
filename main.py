#!/bin/python
import subprocess
import re
from pathlib import Path
import dateutil.parser
import argparse

date_regex = re.compile(r"^.*(20[0-9]{2}[-_]?[0-9]{2}[-_]?[0-9]{2}[-_]?([0-9]{6}|[0-9]{2}[-_][0-9]{2}[-_][0-9]{2})).*$")
whatsapp_regex = re.compile(r"^.*(20\d{6})-WA.*$")
extension_choices = ["jpeg", "jpg", "JPG", "mp4", "MP4"]

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("-d", "--dry-run", action="store_true", default=False, help="show what would be executed. Nothing will be changed")
    parser.add_argument("-e", "--extension", action="append", choices=extension_choices, default=extension_choices)
    parser.add_argument("directory", type=str)

    args = parser.parse_args()

    path = Path(args.directory)

    if not path.exists():
        print(f"Directory '{args.directory}' does not exist")
        exit(1)

    for ext in args.extension:
        for file in path.rglob(f"*.{ext}"):
            date = None
            if match := date_regex.search(file.name):
                date = match.groups()[0]
            elif match := whatsapp_regex.search(file.name):
                date = match.groups()[0]

            try:
                date = re.sub("_", "-", date)
                date = dateutil.parser.parse(date)
                command = f'exiftool -overwrite_original -AllDates="{date}" "{file.absolute()}"'
                if args.dry_run:
                    print(command)
                else:
                    subprocess.run(command, shell=True, check=True)
            except Exception as ex:
                print(f"error parsing date from '{file}': {ex}")
                pass

if __name__ == "__main__":
    main()
