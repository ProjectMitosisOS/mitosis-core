import os

def main():
    print("hello from python: pid: %d ppid: %d" % (os.getpid(), os.getppid()))

if __name__ == '__main__':
    main()
