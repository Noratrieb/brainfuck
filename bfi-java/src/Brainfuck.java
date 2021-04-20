import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Collectors;

public class Brainfuck {

    private static final int MEM_SIZE = 0xFFFF;

    public List<Character> minify(String program) {
        List<Character> chars = List.of('>', '<', '+', '-', '.', ',', '[', ']');
        return program.chars()
                .mapToObj(c -> (char) c)
                .filter(chars::contains)
                .collect(Collectors.toList());
    }


    public String interpret(List<Character> pgm) {
        StringBuilder out = new StringBuilder();
        int pointer = 0;
        short[] memory = new short[MEM_SIZE];
        int pc = 0;

        while (pc < pgm.size()) {
            switch (pgm.get(pc)) {
                case '>' -> {
                    if (pointer == MEM_SIZE - 1) {
                        pointer = 0;
                    } else {
                        pointer++;
                    }
                }
                case '<' -> {
                    if (pointer == 0) {
                        pointer = MEM_SIZE - 1;
                    } else {
                        pointer--;
                    }
                }
                case '+' -> increment(memory, pointer);
                case '-' -> decrement(memory, pointer);
                case '.' -> {
                    out.append((char) memory[pointer]);
                }
                case ',' -> {
                } //todo implement i guess
                case '[' -> {
                    if (memory[pointer] == 0) {
                        int level = 0;
                        while (pgm.get(pc) != ']' || level > -1) {
                            pc++;
                            char instruction = pgm.get(pc);
                            if (instruction == '[') level++;
                            else if (instruction == ']') level--;
                        }
                    }
                }
                case ']' -> { //error lies here
                    if (memory[pointer] != 0) {
                        int level = 0;
                        while (pgm.get(pc) != '[' || level > -1) {
                            pc--;
                            char instruction = pgm.get(pc);
                            if (instruction == '[') level--;
                            else if (instruction == ']') level++;
                        }
                    }
                }
            }

            pc++;
        }

        return out.toString();
    }

    private void increment(short[] memory, int pointer) {
        if (memory[pointer] == 0xFF) {
            memory[pointer] = 0;
        } else {
            memory[pointer]++;
        }
    }

    private void decrement(short[] memory, int pointer) {
        if (memory[pointer] == 0) {
            memory[pointer] = 0xFF;
        } else {
            memory[pointer]--;
        }
    }

    public static void main(String[] args) throws IOException {
        Brainfuck brainfuck = new Brainfuck();

        if (args.length < 1) {
            System.out.println("Please specify a path");
            return;
        }

        String program = Files.readString(Paths.get(args[0]));
        List<Character> minified = brainfuck.minify(program);
        long time1 = System.currentTimeMillis();
        String result = brainfuck.interpret(minified);
        long time = System.currentTimeMillis() - time1;
        System.out.println(result);
        System.out.println("Finished execution in " + time + "ms");
    }
}
