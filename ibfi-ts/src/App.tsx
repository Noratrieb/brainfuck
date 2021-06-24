import './App.scss';
import CodeInput from "./components/CodeInput";
import ProgramOutput from "./components/ProgramOutput";
import React, {useCallback, useState} from "react";
import Runner from "./components/Runner";

function App() {
    const [out, setOut] = useState("");
    const [input, setInput] = useState("");
    const [running, setRunning] = useState(false);

    const outHandler = useCallback((char: number) => {
        setOut(out => out + String.fromCharCode(char))
    }, []);

    const inHandler = useCallback((): number => {
        return 65;
    }, []);

    const runHandler = (run: boolean) => {
        setRunning(run);
        if (!run) {
            setOut("");
        }
    }

    return (
        <div className="App-header">
            {
                !running && <CodeInput setInput={input => setInput(input)}/>
            }
            <Runner running={running} setRunning={runHandler} input={input} outHandler={outHandler} inHandler={inHandler}/>
            {
                running && <ProgramOutput text={out}/>
            }
        </div>
    );
}

export default App;