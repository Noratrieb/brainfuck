import React, {useRef, useState} from 'react';
import Interpreter from "../brainfuck/Interpreter";

const MAX_TABLE_COLUMNS = 20;

interface RunDisplayProps {
    interpreter: Interpreter,
}

const RunDisplay = ({interpreter}: RunDisplayProps) => {

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

const MemoryCell = ({index, interpreter}: { index: number, interpreter: Interpreter }) => {
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

    return (
        <td onClick={click} className="cell">
            {
                isEditing ?
                    <input onKeyDown={keyDown}
                           className="array-set-value-field"
                           ref={inputField}
                           onChange={e => setInput(e.target.value)}
                           value={input}
                           onBlur={saveAndQuit}
                           autoFocus
                    />
                    :
                    interpreter.array[index]
            }
        </td>
    );
}

export default RunDisplay;