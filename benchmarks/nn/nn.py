import math
import random
import time

random.seed(12345)

def xavier(n_in, n_out):
    limit = math.sqrt(6.0 / (n_in + n_out))
    return [[(random.random() * 2 - 1) * limit for _ in range(n_out)] for _ in range(n_in)]

class Layer:
    def __init__(self, n_in, n_out, act):
        self.W = xavier(n_in, n_out)
        self.b = [0.0] * n_out
        self.act = act
        self.last_in = None
        self.last_out = None

    def forward(self, x):
        self.last_in = x
        z = []
        y = []
        for j in range(len(self.b)):
            s = self.b[j]
            for i in range(len(x)):
                s += x[i] * self.W[i][j]
            if self.act == "relu":
                o = s if s > 0 else 0.0
            else:
                o = 1.0 / (1.0 + math.exp(-s))
            z.append(s)
            y.append(o)
        self.last_out = y
        self.last_z = z
        return y

    def backward(self, gout, lr, gin=None):
        gz = []
        for j, g in enumerate(gout):
            y = self.last_out[j]
            if self.act == "relu":
                da = 1.0 if y > 0 else 0.0
            else:
                da = y * (1.0 - y)
            gz.append(g * da)
        if gin is not None:
            for i in range(len(self.last_in)):
                s = 0.0
                for j in range(len(gz)):
                    s += gz[j] * self.W[i][j]
                gin[i] = s
        for i in range(len(self.last_in)):
            for j in range(len(gz)):
                self.W[i][j] -= lr * self.last_in[i] * gz[j]
        for j in range(len(gz)):
            self.b[j] -= lr * gz[j]

def train():
    L1 = Layer(2, 4, "relu")
    L2 = Layer(4, 1, "sigmoid")
    X = [[0, 0], [0, 1], [1, 0], [1, 1]]
    Y = [0, 1, 1, 0]
    lr = 0.5
    epochs = 10000
    start = time.perf_counter()
    for epoch in range(epochs):
        loss = 0.0
        for k in range(4):
            h = L1.forward(X[k])
            y = L2.forward(h)
            err = y[0] - Y[k]
            loss += err * err
            gin = [0.0] * 4
            L2.backward([err], lr, gin)
            L1.backward(gin, lr, None)
    end = time.perf_counter()
    print(f"{(end - start) * 1000:.2f}")

if __name__ == "__main__":
    train()
