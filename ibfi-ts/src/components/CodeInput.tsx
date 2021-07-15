import React, {ChangeEvent, useState} from 'react';
import presets from "../presets.json";

export interface CodeOptions {
    minify?: boolean,
    directStart?: boolean,
    startSuperSpeed?: boolean,
    enableBreakpoints?: boolean
    asciiView?: boolean
}

interface CodeInputProps {
    setInput: ((code: string, options: CodeOptions) => void),
    input: [string, CodeOptions]
}

const codeOptions: Array<[string, keyof CodeOptions]> = [
    ["Minify Code", "minify"],
    ["Start directly", "directStart"],
    ["Start in blocking mode", "startSuperSpeed"],
    ["Breakpoints (•)", "enableBreakpoints"],
    ["Show ASCII in memory", "asciiView"]
]

const CodeInput = ({input: [code, options], setInput}: CodeInputProps) => {
    const [fontSize, setFontSize] = useState(40);

    const setPreset = (name: keyof typeof presets) => () => {
        setInput(presets[name], options);
    }

    const changeHandler = (name: keyof CodeOptions) => (event: ChangeEvent<HTMLInputElement>) => {
        setInput(code, {...options, [name]: event.target.checked})
    }

    return (
        <div className="bf-input">
            <div className="code-options-wrapper">
                <div>
                    <label htmlFor="bf-input-fontsize-range">Font Size</label>
                    <input type="range" id="bf-input-fontsize-range" onChange={v => setFontSize(+v.target.value)}/>
                </div>

                {codeOptions.map(([display, id]) =>
                    <CodeOption displayName={display} name={id} options={options} onChange={changeHandler}/>
                )}

            </div>
            <textarea value={code} onChange={e => setInput(e.target.value, options)} style={{fontSize}}
                      className="code-input"
                      placeholder="Input your code here..."/>
            <div>
                <div>Presets</div>
                <div>
                    <button onClick={setPreset("helloworld")}>Hello World</button>
                    <button onClick={setPreset("hanoi")}>Towers of Hanoi</button>
                    <button onClick={setPreset("quine")}>Quine</button>
                    <button onClick={setPreset("gameoflife")}>Game Of Life</button>
                    <button onClick={setPreset("benchmark")}>Benchmark</button>
                    <button onClick={setPreset("fizzbuzz")}>Fizzbuzz</button>
                </div>
            </div>
        </div>
    );
};

interface CodeOptionProps {
    displayName: string,
    name: keyof CodeOptions,
    options: CodeOptions,
    onChange: (name: keyof CodeOptions) => (event: ChangeEvent<HTMLInputElement>) => void,
}

const CodeOption = ({displayName, name, options, onChange}: CodeOptionProps) => (
    <span>
        <input type="checkbox" checked={options[name]} id={`input-options-${name}`}
               onChange={onChange(name)}/>
        <label htmlFor={`input-options-${name}`}>{displayName}</label>
    </span>
);


export default CodeInput;