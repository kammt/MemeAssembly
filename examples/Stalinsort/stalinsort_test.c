#include <stdio.h>
#include <stdint.h>

int array[] = {1, 5, 2, 8, 9, 3, 4, 12, 14, 69, 420, 0};
uint8_t arraySize = sizeof array / sizeof(int);

extern void stalinsort(int array[], uint8_t *arraySize);

int main() {
    printf("Before sorting:\n");
    for(uint8_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }    
    
    stalinsort(array, &arraySize);
    
    printf("\nAfter sorting:\n");
    for(uint8_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }
    printf("\n");
}    
