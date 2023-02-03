#!/usr/bin/env python3

import os
import subprocess

if __name__ == "__main__":
    for item in os.listdir("."):
        if item[-3:] == ".po":
            print("Generating binary translation file from", item)
            subprocess.run(["msgfmt", "-c", "-o", item[:-2]+"mo", item])
        else:
            print("Ignoring", item)

