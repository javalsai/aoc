const fs = require('fs');
const readline = require('readline');

async function processLineByLine() {
  const fileStream = fs.createReadStream('input.txt');

  const rl = readline.createInterface({
    input: fileStream,
    crlfDelay: Infinity
  });

    let t = 0
  for await (const line of rl) {
      let m = line.match(/mul\([0-9]{1,3}, ?[0-9]{1,3}\)/g);
      m = m.map(e => e.split('(')[1].split(')')[0].split(','))
        .forEach(([a,b]) => t+=-a*-b)
  }
  console.log(t);
}

processLineByLine();
