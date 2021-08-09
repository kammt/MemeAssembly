#include <stdio.h>

int array[] = {1, 5, 2, 8, 9, 3, 4, 12, 14, 69, 420, 0};
size_t arraySize = sizeof array / sizeof(int);

extern void stalinsort(int array[], size_t *arraySize);

int main() {
    printf("Before sorting:\n");
    for(size_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }    
    
    stalinsort(array, &arraySize);
    
    printf("\nAfter sorting:\n");
    for(size_t i = 0; i < arraySize; i++) {
        printf("%d,", array[i]);
    }
    printf("\n");
}    
