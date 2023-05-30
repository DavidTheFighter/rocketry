import os

def main():
    debug_output = os.popen('cargo size -- -A -x').read()
    release_output = os.popen('cargo size --release -- -A -x').read()

    print()
    print("\t\t\tFlash\t\tRAM")

    (flash_size, ram_size) = count_size(debug_output)
    print("Debug binary size:\t{}KiB\t\t{}KiB".format(flash_size // 1024, ram_size // 1024))

    (flash_size, ram_size) = count_size(release_output)
    print("Release binary size:\t{}KiB\t\t{}KiB".format(flash_size // 1024, ram_size // 1024))

    print("Chip total:\t\t1024KiB\t\t192KiB")

def count_size(output):
    ram_total = 0
    flash_total = 0

    start = output.find('section')
    output = output[start:]

    for line in output.split('\n'):
        columns = line.split()
        if len(columns) == 0:
            continue
        elif columns[0] == 'section' or columns[0] == 'Total':
            continue
        elif columns[1] == '0':
            continue

        if columns[0] in ['.text', '.bss', '.uninit']:
            ram_total += int(columns[1][2:], base=16)

            if columns[0] == '.text':
                flash_total += int(columns[1][2:], base=16)
        elif columns[0] in ['.vector_table', '.rodata', '.data']:
            flash_total += int(columns[1][2:], base=16)

    return (flash_total, ram_total)


if __name__ == '__main__':
    main()