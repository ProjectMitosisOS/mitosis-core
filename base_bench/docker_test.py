import argparse
import subprocess
import time

def exec(cmd):
    result = subprocess.run(cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    return result.stdout.strip()

def main():
    parser = argparse.ArgumentParser(description="Run Docker benchmark tests")
    parser.add_argument("--image", required=True, help="Docker image to test")
    parser.add_argument("-n", type=int, required=True, help="Number of containers to run")
    
    args = parser.parse_args()
    
    image_name = args.image
    container_count = args.n

    container_ids = []
    print("start docker run containers")
    start_time = time.time()

    v = [0]*container_count
    total_ms = 0.0

    for container_id in range(container_count):
        now = time.time()
        start_command = f"docker run -d {image_name}"
        output = exec(start_command)

        container_ids.append(output.strip())

        status_command = f"docker inspect -f {{{{.State.Status}}}} {output.strip()}"
        status = exec(status_command)

        while "running" not in status and "exited" not in status:
            status = exec(status_command)

        now2 = time.time()

        diff = (now2 - now) * 1000
        v[container_id] = diff
        total_ms += diff

    total = int(time.time() - start_time)
    res = total_ms / container_count

    print(f"Start {container_count} containers, total time: {total} sec")

    for id in container_ids:
        stop_command = f"docker stop {id}"
        exec(stop_command)
        rm_command = f"docker rm {id}"
        exec(rm_command)

    print("Finish benchmarking container!")

if __name__ == "__main__":
    main()