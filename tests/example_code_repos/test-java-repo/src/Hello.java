import java.util.Scanner;

class Hello {
    public static void main(String[] args) {
        System.out.println("Hello world!");
        System.out.println("What's your name?");

        Scanner in = new Scanner(System.in); 
        String s = in.nextLine(); 
        System.out.println("Hello " + s + "!"); 

        in.close();
    }
}