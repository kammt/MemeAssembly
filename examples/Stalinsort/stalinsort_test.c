/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021 Tobias Kamm and contributors

MemeAssembly is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

MemeAssembly is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with MemeAssembly. If not, see <https://www.gnu.org/licenses/>.
*/
#include <stdio.h>

int array[] = {1, 5, 2, 8, 9, 3, 4, 12, 14, 69, 420, 0};
size_t arraySize = sizeof array / sizeof(int);

extern void stalinSort(int array[], size_t *arraySize);

int main() {
    printf("Before sorting:\n");
    for(size_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }    
    
    stalinSort(array, &arraySize);
    
    printf("\nAfter sorting:\n");
    for(size_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }
    printf("\n");
}    
