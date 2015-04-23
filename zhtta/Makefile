PROGRAM_NAME = zhtta
PORT = 4414
CONNS = 60
RATE = 60

setup:
	mkdir www

all: $(PROGRAM_NAME)

build:
	rustc $(PROGRAM_NAME).rs

$(PROGRAM_NAME): $(PROGRAM_NAME).rs
	rustc $(PROGRAM_NAME).rs

clean :
	$(RM) $(PROGRAM_NAME)
    
run: ${PROGRAM_NAME}
	./${PROGRAM_NAME}

debug: ${PROGRAM_NAME}
	RUST_LOG=debug ./${PROGRAM_NAME}

generate: www
	dd if=/dev/urandom of=www/5K.bin bs=5k count=1
	dd if=/dev/urandom of=www/5M.bin bs=5k count=1
	dd if=/dev/urandom of=www/10M.bin bs=10M count=1
	dd if=/dev/urandom of=www/20M.bin bs=20M count=1
	dd if=/dev/urandom of=www/40M.bin bs=40M count=1
	dd if=/dev/urandom of=www/80M.bin bs=80M count=1
	dd if=/dev/urandom of=www/512M.bin bs=512M count=1
	tr "\n" "\0" < zhtta-test.txt > zhtta-test-NUL.txt

test:
	httperf --server localhost --port ${PORT} --rate ${RATE} --num-conns ${CONNS} --wlog=n,./zhtta-test-NUL.txt


