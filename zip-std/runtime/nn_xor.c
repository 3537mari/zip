/* ZIP STD runtime: trainable 2-layer MLP for XOR (backprop). */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

typedef struct {
    int in, out, act; /* 1=relu 2=sigmoid */
    double *W, *b;
    double *last_in, *last_out, *last_z;
    double *gW, *gb;
} Layer;

static unsigned g_rng = 12345u;
static unsigned next_rng(void) {
    g_rng = g_rng * 1103515245u + 12345u;
    return g_rng & 0x7fffffffu;
}

static double xavier(int in, int out) {
    double limit = sqrt(6.0 / (double)(in + out));
    double r = ((double)(next_rng() % 2000000) - 1000000.0) / 1000000.0;
    return r * limit;
}

static Layer *layer_new(int in, int out, int act) {
    Layer *L = (Layer *)calloc(1, sizeof(Layer));
    L->in = in;
    L->out = out;
    L->act = act;
    L->W = (double *)calloc((size_t)in * out, sizeof(double));
    L->b = (double *)calloc((size_t)out, sizeof(double));
    L->gW = (double *)calloc((size_t)in * out, sizeof(double));
    L->gb = (double *)calloc((size_t)out, sizeof(double));
    L->last_in = (double *)calloc((size_t)in, sizeof(double));
    L->last_out = (double *)calloc((size_t)out, sizeof(double));
    L->last_z = (double *)calloc((size_t)out, sizeof(double));
    for (int i = 0; i < in * out; i++) L->W[i] = xavier(in, out);
    return L;
}

static double relu(double x) { return x > 0 ? x : 0; }
static double relu_d(double y) { return y > 0 ? 1 : 0; }
static double sigmoid(double x) { return 1.0 / (1.0 + exp(-x)); }
static double sigmoid_d(double y) { return y * (1.0 - y); }

static void forward(Layer *L, const double *x, double *y) {
    memcpy(L->last_in, x, (size_t)L->in * sizeof(double));
    for (int j = 0; j < L->out; j++) {
        double z = L->b[j];
        for (int i = 0; i < L->in; i++) z += x[i] * L->W[i * L->out + j];
        L->last_z[j] = z;
        if (L->act == 1) y[j] = relu(z);
        else if (L->act == 2) y[j] = sigmoid(z);
        else y[j] = z;
        L->last_out[j] = y[j];
    }
}

/* gin_out: dL/dy -> updates W,b; writes dL/dx into gin */
static void backward(Layer *L, const double *gout, double *gin, double lr) {
    double *gz = (double *)calloc((size_t)L->out, sizeof(double));
    for (int j = 0; j < L->out; j++) {
        double da = 1.0;
        if (L->act == 1) da = relu_d(L->last_out[j]);
        else if (L->act == 2) da = sigmoid_d(L->last_out[j]);
        gz[j] = gout[j] * da;
    }
    if (gin) {
        for (int i = 0; i < L->in; i++) {
            double s = 0;
            for (int j = 0; j < L->out; j++) s += gz[j] * L->W[i * L->out + j];
            gin[i] = s;
        }
    }
    for (int i = 0; i < L->in; i++) {
        for (int j = 0; j < L->out; j++) {
            L->gW[i * L->out + j] = L->last_in[i] * gz[j];
            L->W[i * L->out + j] -= lr * L->gW[i * L->out + j];
        }
    }
    for (int j = 0; j < L->out; j++) {
        L->gb[j] = gz[j];
        L->b[j] -= lr * L->gb[j];
    }
    free(gz);
}

static void train_one(Layer *L1, Layer *L2, const double x[2], double target, double lr, double *err2) {
    double h[4], y[1], g2[1], g1[4];
    forward(L1, x, h);
    forward(L2, h, y);
    double e = y[0] - target;
    *err2 += e * e;
    g2[0] = 2.0 * e;
    backward(L2, g2, g1, lr);
    backward(L1, g1, NULL, lr);
}

#ifdef ZIP_XOR_MAIN
int main(void) {
    Layer *L1 = layer_new(2, 4, 1);
    Layer *L2 = layer_new(4, 1, 2);
    const double lr = 0.5;
    const int epochs = 10000;
    const double X[4][2] = {{0, 0}, {0, 1}, {1, 0}, {1, 1}};
    const double Y[4] = {0, 1, 1, 0};

    clock_t t0 = clock();
    for (int epoch = 0; epoch < epochs; epoch++) {
        double loss = 0;
        for (int k = 0; k < 4; k++) train_one(L1, L2, X[k], Y[k], lr, &loss);
        if (epoch % 1000 == 0) {
            printf("Epoha %d oshibka: %.6f\n", epoch, loss);
        }
    }
    clock_t t1 = clock();

    printf("=== REZULTAT ===\n");
    for (int k = 0; k < 4; k++) {
        double h[4], y[1];
        forward(L1, X[k], h);
        forward(L2, h, y);
        printf("[%g,%g] -> %.4f (target %.0f)\n", X[k][0], X[k][1], y[0], Y[k]);
    }
    printf("train_ms: %ld\n", (long)((t1 - t0) * 1000.0 / CLOCKS_PER_SEC));
    return 0;
}
#endif
