
OTHER_STATIC_LIB := ReadStat/src/libReadStat.a
WIN_STATIC_LIB := ReadStat/src/ReadStat.lib

SOURCES := $(wildcard ReadStat/src/*.c ReadStat/src/sas/*.c ReadStat/src/spss/*.c ReadStat/src/stata/*.c)
OBJECTS = $(SOURCES:.c=.o)

CC := clang
INC := -IReadStat
CCFLAGS := -DNDEBUG $(INC) -DHAVE_ZLIB -g -O2 -Wall -std=c99

.PHONY: all winodws clean

all : clean $(OBJECTS)
	ar rcs $(OTHER_STATIC_LIB) $(OBJECTS)

windows : clean $(OBJECTS)
	ar rcs $(WIN_STATIC_LIB) $(OBJECTS)

%.o : %.c
	$(CC) $(CCFLAGS) -c $< -o $@

clean :
	-rm ReadStat/src/*.o ReadStat/src/sas/*.o ReadStat/src/spss/*.o ReadStat/src/stata/*.o $(STATIC_LIB) -f 2> /dev/null

