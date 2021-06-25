import {CodeOptions} from "../components/CodeInput";

type InHandler = (() => number);
type OutHandler = ((char: number) => void);

export default class Interpreter {
    private readonly _array: Uint8Array;
    private _pointer: number;
    private readonly _code: string;
    private _programCounter: number;

    private readonly _inHandler: InHandler;
    private readonly _outHandler: OutHandler;

    private readonly _options: CodeOptions;

    constructor(input: [string, CodeOptions], outHandler: OutHandler, inHandler: InHandler) {
        const buf = new ArrayBuffer(32000);
        this._array = new Uint8Array(buf);
        this._pointer = 0;

        this._options = input[1];
        if (input[1].minify) {
            this._code = this.minify(input[0])
        } else {
            this._code = input[0];
        }

        this._programCounter = 0;
        this._inHandler = inHandler;
        this._outHandler = outHandler;
    }

    public next() {
        this.execute(this._code[this._programCounter++]);
    }

    public execute(char: string) {
        switch (char) {
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
                    throw new Error("Cannot wrap left");
                }
                this._pointer--;
                break;
            case '.':
                this._outHandler(this.value);
                break;
            case ',':
                this.input();
                break;
            case '[':
                this.loopForwards();
                break;
            case ']':
                this.loopBackwards();
                break;
            case '•':
                if (this._options.enableBreakpoints) {
                    throw new Error("Breakpoint reached");
                }
                break;
            case undefined:
                this._programCounter = this._code.length;
                break;
            default:
                break;
        }
    }

    private loopForwards() {
        if (this.value === 0) {
            let level = 0;
            while (this.lastInstruction !== ']' || level > -1) {
                this._programCounter++;
                if (this._programCounter > this._code.length) {
                    throw new Error("Reached end of code while searching ']'");
                }
                if (this.lastInstruction === '[') level++;
                else if (this.lastInstruction === ']') level--;
            }
        }
    }

    private loopBackwards() {
        if (this.value !== 0) {
            let level = 0;
            while (this.lastInstruction !== '[' || level > -1) {
                this._programCounter--;
                if (this._programCounter < 0) {
                    throw new Error("Reached start of code while searching '['");
                }
                if (this.lastInstruction === '[') level--;
                else if (this.lastInstruction === ']') level++;
            }
        }
    }


    private input() {
        try {
            this._array[this._pointer] = this._inHandler();
        } catch {
            this._programCounter--;
        }
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

    private minify(code: string): string {
        const CHARS = ['+', '-', '<', '>', '.', ',', '[', ']'];
        if (this._options.enableBreakpoints) {
            CHARS.push('•');
        }

        return code.split("")
            .filter(c => CHARS.includes(c))
            .join("");
    }
}



