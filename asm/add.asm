.data
        temp = 7
        temp_1 = 2
        temp_2 = 11
.program
@loop_start
        ADD R0 temp_1 temp_2
        SUB R1 R0 100
        JGT R1 @loop_end
        JMP 0 @loop_start
@loop_end
