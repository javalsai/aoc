const args = Bun.argv.slice(2)

if(args.length !== 1) {
    console.error('Please, provide an input file')
    process.exit(1)
}

const filename = args[0];


const input = Bun.file(filename)
const text = await input.text()

let count = 0
let prevNums: Array<[string, number]> = []
let prevSyms: Array<[number, string[]]> = []
text.split('\n').forEach(ln => {
    const line = ln.trim()
    if(!line) return

    let tokens = line.matchAll(/\d+|\*/g)

    let nums: Array<[string, number]> = [], syms: Array<[number, string[]]> = []
    for(const token of tokens) {
        /\d/.test(token.toString()) ?
            nums.push([token.toString(), token.index as number]) :
            syms.push([token.index as number, []])
    }


    pushNums(prevSyms, nums)
        .map(([_index, attached]) => attached)
        .filter(attached => attached.length == 2)
        .forEach(attached => {
            count += attached
                .map(e => parseInt(e))
                .reduce((a = 1, b = 1) => a * b, 1)
        })

    syms = pushNums(syms, nums)
    syms = pushNums(syms, prevNums)

    prevNums = nums
    prevSyms = syms
})

console.log(count)

function pushNums(symbols: Array<[number, string[]]>, nums: Array<[string, number]>): [number, string[]][] {
    return symbols.map(([index, attached]) => {
        attached.push(
            ...nums.filter(([num, nindex]) => {
                return index >= nindex - 1 && index <= nindex + num.length
            }).map(([num]) => num)
        )
        return [index, attached]
    })
}
