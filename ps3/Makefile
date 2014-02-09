PROGRAM_NAME = zhtta
STARTING_CODE = zhtta-starting-code

all: $(PROGRAM_NAME)

$(PROGRAM_NAME): $(PROGRAM_NAME).rs
	rustc $(PROGRAM_NAME).rs

$(STARTING_CODE): $(STARTING_CODE).rs
	rustc $(STARTING_CODE).rs

clean :
	$(RM) $(PROGRAM_NAME)
    
run: ${PROGRAM_NAME}
	./${PROGRAM_NAME}

start: ${STARTING_CODE}
	./${STARTING_CODE}
