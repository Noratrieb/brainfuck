import './App.scss';
import CodeInput from "./components/CodeInput";
import ProgramOutput from "./components/ProgramOutput";
import React, {useState} from "react";
import Interpreter from "./brainfuck/Interpreter";
import RunInfo from "./components/RunInfo";

function App() {
    const [interpreter, setInterpreter] = useState<Interpreter | null>(null);

    const [out, setOut] = useState("");
    const [input, setInput] = useState("");

    const outHandler = (char: number) => {
        setOut(out => out + String.fromCharCode(char))
    }

    const inHandler = (): number => {
        return 65;
    }

    const start = () => setInterpreter(new Interpreter(input, outHandler, inHandler));
    const next = () => interpreter?.next();
    const prev = () => interpreter?.prev();

    return (
        <div className="App-header">
            <CodeInput setInput={input => setInput(input)}/>
            <RunInfo nextHandler={next} prevHandler={prev} startHandler={start} interpreter={interpreter}/>
            <ProgramOutput text={out}/>
        </div>
    );
}

export default App;