import shutil

time = 0


def lambda_handler():
    global time
    dst = 'result-' + str(time)
    src = 'compression-' + str(time)
    shutil.make_archive(dst, 'zip', src)
    time += 1


if __name__ == '__main__':
    lambda_handler()
    lambda_handler()
    lambda_handler()

