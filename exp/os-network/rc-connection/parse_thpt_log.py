import sys
import re

thpt = []

def avg(thpt):
    sum = 0
    for i in thpt:
        sum += i
    return sum/len(thpt)

def main():
    with open(sys.argv[1], 'r') as f:
        while True:
            line = f.readline()
            if not line:
                break
            res = re.search(r'thpt *: (.*)', line)
            if res:
                thpt.append(float(res.group(1)))
    print(avg(thpt))

if __name__ == '__main__':
    main()
