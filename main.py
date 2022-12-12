#!/bin/python
import subprocess
from datetime import datetime
import re
import sys
from pathlib import Path
import dateutil.parser 

date_re1 = re.compile(r"^.*(20[0-9]{2}[-_]?[0-9]{2}[-_]?[0-9]{2}[-_]?([0-9]{6}|[0-9]{2}[-_][0-9]{2}[-_][0-9]{2})).*$")
whatsapp = re.compile(r"^.*(20\d{6})-WA.*$")

path = Path(sys.argv[1])
for file in path.rglob("*.jpeg"):
    date = None
    if match := date_re1.search(file.name):
        date = match.groups()[0]
    elif match := whatsapp.search(file.name):
        date = match.groups()[0]
    else:
        pass
    try:
        date = re.sub("_", "-",date)
        date = dateutil.parser.parse(date)
        command = f'exiftool -overwrite_original -AllDates="{date}" "{file.absolute()}"'
        #subprocess.run(command, shell=True, check=True)
        print(command)
    except:
        pass
