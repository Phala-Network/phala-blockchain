pub trait MsgTrait<A,B,C,D,E> {
    fn push(&mut self,a:A);
    fn received(&mut self,b:B);
    fn get_msgs(&mut self,c:C)->D;
    fn get_sequence(&mut self)->E;
    fn hello(&mut self){
        println!("hello,world");
    }
}