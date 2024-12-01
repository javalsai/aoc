const args = Bun.argv.slice(2)

if(args.length !== 1) {
    console.error('Please, provide an input file');
    process.exit(1);
}

const filename = args[0];


const input = Bun.file(filename)
const text = await input.text()

const start = new Date();
const result = text.split('\n').map(e => e.trim()).map(line => {
    if(!line) return;

    const match = matchOverlap(line, /(?:[0-9]|one|two|three|four|five|six|seven|eight|nine)/gi) ?? []
    console.log(match)
    const [leftmost, rightmost] = [match[0] ?? 0, match.slice(-1)[0] ?? 0];

    console.log(line, leftmost, rightmost, toNum(leftmost), toNum(rightmost))

    return toNum(leftmost) * 10 + toNum(rightmost);
}).reduce((a = 0, b = 0) => a + b, 0)
const end = new Date();
console.log(result)
console.log(end - start);

function toNum(n: number | string): number {
    if(typeof n == 'number') return n;
    else {
        switch(n.toLowerCase()) {
            case 'one':
                return 1;
            case 'two':
                return 2;
            case 'three':
                return 3;
            case 'four':
                return 4;
            case 'five':
                return 5;
            case 'six':
                return 6;
            case 'seven':
                return 7;
            case 'eight':
                return 8;
            case 'nine':
                return 9;
            default:
                return parseInt(n)
        }
    }
}

function matchOverlap(input, re) {
    var r = [], m;
    // Prevent infinite loops
    if (!re.global) re = new RegExp(
        re.source, (re+'').split('/').pop() + 'g'
    );
    while (m = re.exec(input)) {
        re.lastIndex -= m[0].length - 1;
        r.push(m[0]);
    }
    return r;
}

