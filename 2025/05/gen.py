import random

MAX=(2**64)-1
RANGES=1000000
IDS=100000000

list_of_ids = []

with open('2025-huge.txt', 'a') as file:
    for x in range(1000):
        num1 = random.randint(1, MAX)
        num2 = random.randint(1, MAX)
        if num1 > num2:
            tmp = num1
            num1 = num2
            num2 = tmp
        file.write(f'{num1}-{num2}\n')
    file.write('\n')
    for x in range(IDS):
        num = random.randint(2, MAX-1)
        if not (num in list_of_ids):
            file.write(f'{num}\n')
