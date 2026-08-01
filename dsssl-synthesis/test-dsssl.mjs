// dsssl-synthesis/test-dsssl.mjs
// Runs the DSSSL kernel on the embedded example and verifies the output.

import { runDSSL } from './dsssl-kernel.mjs'

const SGML = `<GROVE>
  <EXPR id="root">
    <OP>*</OP>
    <LEFT>
      <EXPR id="inner">
        <OP>+</OP>
        <LEFT><HOLE var="?x"></LEFT>
        <RIGHT><INT>4</INT></RIGHT>
      </EXPR>
    </LEFT>
    <RIGHT><INT>4</INT></RIGHT>
  </EXPR>
</GROVE>`

async function test() {
  const result = await runDSSL(SGML)

  console.log('DSSSL Kernel Test')
  console.log('Input:  (?x + 4) * 4 = 20')
  console.log('Target: 20')
  console.log()

  // ?x = 1: (1 + 4) * 4 = 5 * 4 = 20  ✓
  console.log('Expected: ?x = 1, result = 20')
  console.log('Got:     ', result.synthesis?.candidate, result.synthesis?.result)
  console.log('Status:  ', result.status)
  console.log()

  if (result.status !== 'VERIFIED') {
    console.error('FAIL: synthesis did not verify')
    process.exit(1)
  }
  if (result.synthesis?.candidate !== 1) {
    console.error('FAIL: expected candidate 1, got', result.synthesis?.candidate)
    process.exit(1)
  }
  if (result.synthesis?.result !== 20) {
    console.error('FAIL: expected result 20, got', result.synthesis?.result)
    process.exit(1)
  }

  console.log('✓ DSSSL kernel: hole resolved, invariant satisfied, grove verified')
  console.log('✓ WORM seal:', result.worm_seal)
  console.log()
  console.log('Output grove:')
  console.log(result.output_grove_sgml)
}

test().catch(e => { console.error(e); process.exit(1) })
