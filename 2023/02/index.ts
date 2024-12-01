const args = Bun.argv.slice(2)

if(args.length !== 1) {
    console.error('Please, provide an input file')
    process.exit(1)
}

const filename = args[0];


const input = Bun.file(filename)
const text = await input.text()

let count = 0
text.split('\n').map(e => e.trim()).filter(line => line)
    .forEach(line => {
        const [gameID, gameData] = line.split(':')
        const min = gameData.split(';')
            .map(set => set.trim())
            .map(set => addArraysToObj(
                    set.split(',').map(color =>
                        color.trim().split(' ').reverse() as [string, string]
                    )
                )
            ).reduce((acc, object) => {
                for(const key of Object.keys(object)) {
                    acc[key] = Math.max(acc[key] ?? 0, object[key] ?? 0)
                }
                return acc
            }, {})

        count += Object.values(min).reduce((a=1, b=1) => a * b, 1)
    })

console.log(count)

function addArraysToObj(array: Array<[string, number | string]>) {
    const object: { [key: string]: number } = {}
    array.forEach(([key, val]) => object[key] = (object[key] ?? 0) + toNum(val))

    return object
}


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
