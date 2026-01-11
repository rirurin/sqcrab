function test1() {
    return 10;
}

function square(n) {
    return n * n;
}

function fib(n) {
    if (n < 2) return 1;
    return fib(n - 2) + fib(n - 1);
}

function ack(m, n) {
    if (m == 0) return (n + 1);
    if (n == 0) return ack(m - 1, 1);
    return(ack(m - 1, ack(m, (n - 1))) );
}