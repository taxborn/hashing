import time
import hashlib

BEST_SHA = hashlib.sha256("test/0".encode('utf-8')).hexdigest()
START_IDX = 0

def leading_zeroes(input: str):
    count = 0

    while count < len(input) and input[count] == '0':
        count += 1

    return count

if __name__ == "__main__":
    best = str(BEST_SHA)
    start = time.time()

    for i in range(START_IDX, int(1e14)):
        submission = f"taxborn/i7+13700K/{i}"
        computed_hash = hashlib.sha256(submission.encode('utf-8')).hexdigest()

        # Every 1b, print the index we are at
        if i % 1_000_000_000 == 0: print(i)

        if computed_hash < best:
            print(f"{submission:>48}", end=" ")
            with open('out.txt', 'a') as file:
                file.write(f"{submission}: {computed_hash}\n")
            best = computed_hash
            dur = time.time() - start
            print(f"{i / dur / 1000 / 1000:3.2f} MH/s", end=" ")

            print(computed_hash, f"{leading_zeroes(computed_hash) = } (#1 is 13)")

