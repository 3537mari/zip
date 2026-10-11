import time
import sys
sys.setrecursionlimit(10000)

def fibonacci(n):
    if n < 2:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

if __name__ == "__main__":
    n = 40
    start = time.perf_counter()
    result = fibonacci(n)
    end = time.perf_counter()
    print(result)
    print(f"{(end - start) * 1000:.2f}")
