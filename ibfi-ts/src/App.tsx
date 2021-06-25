import './App.scss';
import CodeInput, {CodeOptions} from "./components/CodeInput";
import ProgramOutput from "./components/ProgramOutput";
import React, {useCallback, useState} from "react";
import Runner from "./components/Runner";

function App() {
    const [out, setOut] = useState("");
    const [input, setInput] = useState<[string, CodeOptions]>(["", {}]);
    const [running, setRunning] = useState(false);

    const outHandler = useCallback((char: number) => {
        setOut(oldOut => oldOut + String.fromCharCode(char))
    }, []);

    const runHandler = (run: boolean) => {
        setRunning(run);
        if (!run) {
            setOut("");
        }
    }

    const inputHandler = (code: string, options: CodeOptions) => setInput([code, options]);
    return (
        <div className="App-header">
            {
                !running && <CodeInput code={input[0]} setInput={inputHandler}/>
            }
            <Runner running={running} setRunning={runHandler} input={input} outHandler={outHandler}/>
            {
                running && <ProgramOutput text={out}/>
            }
        </div>
    );
}

export default App;