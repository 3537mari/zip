import json
import time

data = {"данные": [{"id": i} for i in range(10000)]}
json_str = json.dumps(data, ensure_ascii=False)

start = time.perf_counter()
parsed = json.loads(json_str)
end = time.perf_counter()
_ = len(parsed["данные"])
print(f"{(end - start) * 1000:.2f}")
