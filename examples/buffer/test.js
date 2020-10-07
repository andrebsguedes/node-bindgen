const assert = require('assert');

let addon = require('./dist');
const { format } = require('path');

let bytes = addon.test(5);
console.log(bytes)
console.log("received bytes: ",bytes.length);

console.log(bytes.toString())

console.log('Good 1')

// create buffer view from byte array
let buffer = bytes;
assert.deepEqual(JSON.parse(buffer), { a: 'b', b: 5});

console.log('Good 2')

let buff = Buffer.from([1, 2, 3, 4])

console.log(buff)

addon.test1(buff);

let record = addon.test2(10);
assert.equal(record.comment,"array buffer is cool!");
assert.deepEqual(JSON.parse(Buffer.from(record.buffer)), { a: 'b', b: 10});

