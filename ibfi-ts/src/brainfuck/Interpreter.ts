import {CodeOptions} from "../components/CodeInput";

type InHandler = (() => number);
type OutHandler = ((char: number) => void);
type ErrorHandler = ((msg: string) => void);

export default class Interpreter {
    private readonly _array: Uint8Array;
    private _pointer: number;
    private readonly _code: string;
    private _programCounter: number;

    private readonly _inHandler: InHandler;
    private readonly _outHandler: OutHandler;
    private readonly _errorHandler: ErrorHandler;

    constructor(input: [string, CodeOptions], outHandler: OutHandler, inHandler: InHandler, errorHandler: ErrorHandler) {
        const buf = new ArrayBuffer(32000);
        this._array = new Uint8Array(buf);
        this._pointer = 0;

        if (input[1].minify) {
            this._code = minify(input[0])
        } else {
            this._code = input[0];
        }

        this._programCounter = 0;
        this._inHandler = inHandler;
        this._outHandler = outHandler;
        this._errorHandler = errorHandler;
    }

    public next() {
        switch (this._code[this._programCounter++]) {
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
                if (this._pointer === 0) {
                    this._errorHandler("Cannot wrap left");
                    break;
                }
                this._pointer--;
                break;
            case '.':
                this._outHandler(this.value);
                break;
            case ',':
                try {
                    this._array[this._pointer] = this._inHandler();
                } catch {
                    this._programCounter--;
                    this._errorHandler("Could not read input, trying again next time.")
                }
                break;
            case '[':
                if (this.value === 0) {
                    let level = 0;
                    while (this.lastInstruction !== ']' || level > -1) {
                        this._programCounter++;
                        if (this.lastInstruction === '[') level++;
                        else if (this.lastInstruction === ']') level--;
                    }
                }
                break;
            case ']':
                if (this.value !== 0) {
                    let level = 0;
                    while (this.lastInstruction !== '[' || level > -1) {
                        this._programCounter--;
                        if (this.lastInstruction === '[') level--;
                        else if (this.lastInstruction === ']') level++;
                    }
                }
                break;
            case undefined:
                this._pointer = this._code.length;
                console.warn("reached end");
                break;
            default: {
            }
        }
        console.log(`char: ${this.code[this.programCounter - 1]}  pointer: ${this.pointer} value: ${this.array[this.pointer]}`)
    }

    public prev() {

    }

    get reachedEnd(): boolean {
        return this._programCounter === this._code.length;
    }

    get lastInstruction(): string {
        return this._code[this._programCounter - 1];
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

    get programCounter(): number {
        return this._programCounter;
    }
}

const CHARS = ['+', '-', '<', '>', '.', ',', '[', ']'];
const minify = (code: string): string =>
    code.split("")
    .filter(c => CHARS.includes(c))
    .join("");

