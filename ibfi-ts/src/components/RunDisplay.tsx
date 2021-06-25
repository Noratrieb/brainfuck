import React, {useContext, useRef, useState} from 'react';
import Interpreter from "../brainfuck/Interpreter";
import {OptionContext} from "../App";

const MAX_TABLE_COLUMNS = 20;

interface RunDisplayProps {
    interpreter: Interpreter,
}

const RunDisplay = ({interpreter}: RunDisplayProps) => {
    const options = useContext(OptionContext);

    const index = interpreter.pointer;

    let offset: number;

    if (index < MAX_TABLE_COLUMNS / 2) {
        offset = 0;
    } else {
        offset = index - MAX_TABLE_COLUMNS / 2;
    }

    const arrayWithIndex = Array(MAX_TABLE_COLUMNS).fill(0)
        .map((_, i) => i + offset);

    return (
        <div>
            <table className="memory-display-table">
                <thead>
                <tr>
                    {
                        arrayWithIndex.map((n => <th key={n}>{n}</th>))
                    }
                </tr>
                </thead>
                <tbody>
                <tr>
                    {
                        arrayWithIndex.map((n) => <MemoryCell key={n} index={n} interpreter={interpreter}/>)
                    }
                </tr>
                {
                    options.asciiView &&
                    <tr>
                        {
                            arrayWithIndex.map((n) => <MemoryCell key={n} index={n} interpreter={interpreter} ascii/>)
                        }
                    </tr>
                }
                <tr>
                    {
                        arrayWithIndex.map((n) => <td className="pointer"
                                                      key={n}>{interpreter.pointer === n && "^"}</td>)
                    }
                </tr>
                </tbody>
            </table>
        </div>
    );
};

interface MemoryCellProps {
    index: number,
    interpreter: Interpreter,
    ascii?: boolean,
}

const MemoryCell = ({index, interpreter, ascii}: MemoryCellProps) => {
    const [isEditing, setIsEditing] = useState(false);
    const [input, setInput] = useState(interpreter.array[index] + "");

    const inputField = useRef<HTMLInputElement>(null);

    const saveAndQuit = () => {
        interpreter.array[index] = +(input);
        setIsEditing(false);
    }

    const click = () => {
        setIsEditing(true);
        inputField.current?.select();
    }

    const keyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
        console.log("key", e.key);
        if (e.key === "Escape") {
            setIsEditing(false);
        } else if (e.key === "Enter") {
            saveAndQuit();
        }
    }

    const content = ascii ?
        String.fromCharCode(interpreter.array[index])
        :
        interpreter.array[index];

    return (
        <td onClick={click} className="cell">
            {
                isEditing && !ascii ?
                    <input onKeyDown={keyDown}
                           className="array-set-value-field"
                           ref={inputField}
                           onChange={e => setInput(e.target.value)}
                           value={input}
                           onBlur={saveAndQuit}
                           autoFocus
                    />
                    :
                    content
            }
        </td>
    );
}

export default RunDisplay;