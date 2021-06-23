import React from 'react';
import Interpreter from "../brainfuck/Interpreter";

interface RunInfoProps {
    nextHandler: () => void,
    prevHandler: () => void,
    startHandler: () => void,
    interpreter: Interpreter | null,
}

const RunInfo = ({interpreter, ...props}: RunInfoProps) => {
        return (
            <div className="bf-run">
                <div>
                    <button onClick={props.startHandler}>Start</button>
                    <button onClick={props.nextHandler}>Next</button>
                    <button onClick={props.prevHandler}>Previous</button>
                </div>

                {interpreter &&
                <div>Pointer: {interpreter.pointer} value {interpreter.value}</div>
                }
            </div>
        );
    }
;

export default RunInfo;