start = 100000
end = 999999
count = 0
for i in range(start, end):
    n = i
    double = 0
    prev = 0
    for _ in range(5):
        b = n % 10
        a = n // 10 % 10

        if a > b:
            break
        if a == b:
            if double == a:
                prev = double
                double = 0
            elif prev != a and not double:
                double = a

        n //= 10
    else:
        if double:
            count += 1
print(count)
