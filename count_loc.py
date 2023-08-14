#!/usr/bin/python3

import os

SEARCH_DIRECTORIES = ['design-scripts', 'ecu', 'fcu', 'hal', 'mission-ctrl', 'simulation', 'comms-manager', 'streamish'] # What directories to recursively search
DIRECTORY_BLACKLIST = ['target', 'build', 'node_modules', 'dist', 'assets']
CODE_FILE_WHITELIST = ['.rs', '.py', '.js', '.vue', '.html', '.css']

def getNumberOfLinesInFile(filepath):
    return sum(1 for line in open(filepath, 'r'))

def countLinesOfCodeInRoot(root, lst):
    totalLinesOfCode = 0

    for path, subdirs, files in os.walk(root):
        skip = False
        for blacklist_dir in DIRECTORY_BLACKLIST:
            if blacklist_dir in path.split(os.sep):
                skip = True

        if skip:
            continue

        for filename in files:
            if any(filename.endswith(extension) for extension in CODE_FILE_WHITELIST):
                filepath = os.path.join(path, filename)
                linesOfCode = getNumberOfLinesInFile(filepath)

                lst += [(filepath, linesOfCode)]

                totalLinesOfCode += linesOfCode

    return totalLinesOfCode

def countLinesOfCode():
    totalLinesOfCode = 0
    lst = []

    for dir in SEARCH_DIRECTORIES:
        totalLinesOfCode += countLinesOfCodeInRoot(dir, lst)

    lst.sort(key=lambda x: x[1], reverse=True)
    for filepath, linesOfCode in lst:
        print("\"{}\" has {} lines".format(filepath, linesOfCode))

    print('\nTotal lines of code: {} ({:.2f} kLOC)'.format(totalLinesOfCode, totalLinesOfCode / 1000))

if __name__ == '__main__':
    countLinesOfCode()