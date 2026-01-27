.data
        temp = 7
        ; temp_1 = 2
        ; temp_2 = 11
.program
        ;MOV SUPPORTED THINGS:
        ; MOV temp temp_1
        ; MOV temp 10
        ; MOV temp RAX
        ; MOV temp [RAX]
        ; MOV temp RDX
        ;
        ; MOV RAX temp
        LI RAX 2
        MOV [RAX] temp
        ; MOV RDX temp
