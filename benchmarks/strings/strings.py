import time

N = 1_000_000
start = time.perf_counter()
h = 2166136261
buf = []
length = 0
for i in range(N):
    tmp = f"{i % 1000},"
    buf.append(tmp)
    length += len(tmp)
    for ch in tmp.encode("ascii"):
        h ^= ch
        h = (h * 16777619) & 0xFFFFFFFF
    if length > 4096:
        buf.clear()
        length = 0
end = time.perf_counter()
print(h)
print(f"{(end - start) * 1000:.2f}")
