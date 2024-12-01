// gcc -static main.c -o main -Ofast

#include <stdio.h>
#include <unistd.h>
#include <time.h>

unsigned long long count = 0;

unsigned char start_index;
unsigned char scratches_end;
unsigned char results_index;
unsigned char line_size;

unsigned short k = 0;
//unsigned int sclones = 0;
unsigned long long clones[32767] = { [0 ... 32766] = 1 };
void addCardToCount(char *line) {
    unsigned char matches = 0;
    for(unsigned char i = start_index; i < scratches_end; i += 3) {
        for(unsigned char j = results_index; j < line_size; j += 3) {
            unsigned char match = (line[i] == line[j] && line[i + 1] == line[j + 1]);
            matches += match;
            if(match) break;
        }
    }

    //unsigned long long this_instan = sclones > k ? clones[k] + 1 : 1;
    unsigned long long this_instan = clones[k];
    for(unsigned short i = k + 1; i < matches + k + 1; i++) {
        //clones[i] = ((sclones > i) * clones[i]) + this_instan;
        clones[i] += this_instan;
    }
    //sclones = sclones > k + matches ? sclones + 1 : k + matches;

    count += this_instan;
    k++;
}

int main() {
    clock_t begin = clock();

    unsigned char i = 0;
    char line[256];

    while(read(0, &line[i], 1) > 0 && line[i] != ':') {
        i++;
    }
    start_index = ++i + 1;

    while(read(0, &line[i], 1) > 0 && line[i] != '|') {
        i++;
    }
    scratches_end = i - 1;
    results_index = i++ + 2;

    while(read(0, &line[i], 1) > 0 && line[i] != '\n') {
        i++;
    }
    line_size = ++i;

    do {
        addCardToCount(line);
    } while(read(0, &line, line_size) == line_size);

    clock_t end = clock();
    double time_spent = (double)(end - begin) / CLOCKS_PER_SEC;

    printf("%llu\n", count);
    printf("Time spent: %lf\n", time_spent);

    return 0;
}
