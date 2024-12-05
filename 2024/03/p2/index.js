const fs = require('node:fs');
const readline = require('readline');

async function processLineByLine() {
  const fileStream = fs.createReadStream('../input.txt');

  const rl = readline.createInterface({
    input: fileStream,
    crlfDelay: Infinity
  });

  let t = 0
  let foo_do = true;
  for await (const line of rl) {
    let m = line.match(/(?:mul\([0-9]{1,3}, ?[0-9]{1,3}\)|do(?:n't)?\(\))/g);
    m = m.forEach(e => {
      if(e.startsWith('mul') && foo_do) {
        let [a,b] = e
          .split('(')[1]
          .split(')')[0]
          .split(',')

        t += -a * -b;
      }
      if(e.startsWith('do')) foo_do = true;
      if(e.startsWith('don')) foo_do = false;
    })
  }
  console.log(t);
}

processLineByLine();
