import numpy as np
import time

size = 512
a = np.ones((size, size), dtype=np.float64)
b = np.full((size, size), 2.0, dtype=np.float64)

start = time.perf_counter()
c = a @ b
end = time.perf_counter()

print(c[0][0])
print(f"{(end - start) * 1000:.2f}")
