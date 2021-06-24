export default class Interpreter {
    private readonly _array: Uint8Array;
    private _pointer: number;
    private readonly _code: string;
    private _codePointer: number;

    private readonly _inHandler: (() => number);
    private readonly _outHandler: ((char: number) => void);

    constructor(code: string, outHandler: ((char: number) => void), inHandler: (() => number)) {
        const buf = new ArrayBuffer(32000);
        this._array = new Uint8Array(buf);
        this._pointer = 0;
        this._code = code;
        this._codePointer = 0;
        this._inHandler = inHandler;
        this._outHandler = outHandler;
    }

    public next() {
        switch (this._code[this._codePointer++]) {
            case '+':
                this._array[this._pointer]++;
                break;
            case '-':
                this._array[this._pointer]--;
                break;
            case '>':
                this._pointer++;
                break;
            case '<':
                this._pointer--;
                break;
            case '.':
                this._outHandler(this.value);
                break;
            case ',':
                this._array[this._pointer] = this._inHandler();
                break;
            case '[':
                if (this.value === 0) {
                    let level = 0;
                    while (this.instruction !== ']' || level > -1) {
                        this._codePointer++;
                        if (this.instruction === '[') level++;
                        else if (this.instruction === ']') level--;
                    }
                }
                break;
            case ']':
                if (this.value !== 0) {
                    let level = 0;
                    while (this.instruction !== '[' || level > -1) {
                        this._codePointer--;
                        if (this.instruction === '[') level--;
                        else if (this.instruction === ']') level++;
                    }
                }
                break;
            case undefined:
                console.warn("reached end");
                break;
            default: {
            }
        }
        console.log(`char: ${this.code[this.codePointer - 1]}  pointer: ${this.pointer} value: ${this.array[this.pointer]}`)
    }

    public prev() {

    }

    get instruction(): string {
        return this._code[this._codePointer];
    }

    get value(): number {
        return this._array[this._pointer];
    }

    get array(): Uint8Array {
        return this._array;
    }

    get pointer(): number {
        return this._pointer;
    }

    get code(): string {
        return this._code;
    }

    get codePointer(): number {
        return this._codePointer;
    }
}