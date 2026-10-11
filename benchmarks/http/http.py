import time

REQ = "GET /api/v1/item/42 HTTP/1.1\r\nHost: localhost\r\n\r\n"
N = 5_000_000

def parse_get(req: str) -> int:
    if not req.startswith("GET "):
        return -1
    p = 4
    if req[p] != "/":
        return -1
    while p < len(req) and req[p] != " ":
        p += 1
    return p - 4

start = time.perf_counter()
s = 0
for _ in range(N):
    s += parse_get(REQ)
end = time.perf_counter()
print(s)
print(f"{max((end - start) * 1000, 0.01):.2f}")
