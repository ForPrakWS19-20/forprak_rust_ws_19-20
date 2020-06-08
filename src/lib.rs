#![allow(unused)]
use std::fs::{File, OpenOptions, Metadata, metadata};
use std::io::{SeekFrom, Seek, Read, Write};
use std::{fs, mem};
use std::iter::Map;
use std::collections::HashMap;
use std::time::SystemTime;


// BFA, Block file access, bietet die Moeglichkeit, Block zu get und put
// Ein Block hat eine einzige ID, richtet nach einem Bereich von xxByte nach xxByte in File
// Gebe bestimmte ID, kriege den Block, kriege den Teil v  cxdsaq   c   on File

pub struct BFA{
    pub block_size: usize,
    pub file: File,
    //metadata sollte auf Typ Map sein
    metadaten: HashMap<String,String>,
    //1 for true, 0 for false
    update_file: Vec<bool>,
    reserved_file: Vec<bool>,
    reserve_count:usize
}

pub struct Block{
    pub contents: Vec<u8>,
}

pub struct RTree{
    root_id:usize,
    bfa: BFA,
    dimension:usize,
    total_id: usize
}

enum Node{
    Leaf{
        mbr:MBRect,
        child: block,
        index_zeige:usize
    },
    InnerNode{
        id:usize,
        mbr: MBRect,
        children: Vec<Node>
    },
}

/*pub struct Node {
    leaf: bool,
    content: Vec<usize>,
    rect: MBRect,
    id: usize
}*/

pub struct Point{
    //Pos 0: x, Pos 1: y
    //coor:Vec<f64>
    x:f64,
    y:f64
}

pub struct MBRect{
    num: usize,
    botton_left:Point,
    top_right:Point,
}

impl Point{
    pub fn new(x:f64, y:f64) -> Self{
        Point(x,y)
    }
}

impl MBRect{
    pub fn new(bl:Point, tp:Point, id:usize) -> Self{
        MBRect(id,bl,tp)
    }

    fn mbr_of(p1:Point, p2:Point, id:usize) -> MBRect{
        let min_x = min(p1.coor[0], p2.coor[0]);
        let min_y = min(p1.coor[1], p2.coor[1]);
        let max_x = max(p1.coor[0], p2.coor[0]);
        let max_y = max(p1.coor[1], p2.coor[1]);
        let min: Point = Point::new(min_x,min_y);
        let max: Point = Point::new(max_x,max_y);
        let rect: MBRect = MBRect::new(min,max,id);
        rect
    }

    fn mbr_of_rects(r1:&MBRect, r2:&MBRect, id:usize) -> MBRect{
        let p1 = &r1.top_right;
        let p2 = &r1.botton_left;
        let p3 = &r2.top_right;
        let p4 = &r2.botton_left;
        let min_x = min(p2.coor[0], p4.coor[0]);
        let min_y = min(p2.coor[1], p4.coor[1]);
        let max_x = max(p1.coor[0], p3.coor[0]);
        let max_y = max(p1.coor[1], p3.coor[1]);
        let min: Point = Point::new(min_x,min_y);
        let max: Point = Point::new(max_x,max_y);
        let rect: MBRect = MBRect::new(min,max,id);
        rect
    }

    fn overlap(&self, other:MBRect) -> bool {
        return !((self.max.coor.get(1) < other.min.coor.get(1))
            || (other.max.coor.get(1) < self.min.coor.get(1))
            ||(self.max.coor.get(0) < other.min.coor.get(0))
            ||(other.max.coor.get(0) < self.min.coor.get(0)))
    }
}

impl Node{

    pub fn rect(&self) -> MBRect{

        match node{

            Node::Leaf {mbr, child, index_zeige} => mbr,
            Node::InnerNode { id, mbr, children } => mbr,

        }
    }

    pub fn new(leaf: bool, content: Vec<usize>,  MBRect:MBRect, id:usize) -> Self{
        Node{leaf, content, rect,id}
    }

    pub fn from_block(block: & mut Block) -> Self{
        let node = bincode::deserialize(block.contents.as_slice()).unwrap();
        node
    }
}

impl RTree{
    fn new(mut bfa: BFA, total_id: usize) -> Self{
        let root_id = bfa.get_root();
        let dimension: usize = 2;
        RTree{root_id, bfa, dimension, total_id}
    }

    pub fn node_is_leaf(&mut self, node: &Node) -> bool{
        match node {
            Node::Leaf => true,
            Node::InnerNode => false,
        }
    }

/////////////////////////////////////////////////////////////////////////////////////////

    //search: suchen einen Rect, der mit dem gegebenen Rect ueberlapped
    //search children tree
    //root is not a leaf node
    //search from root,
    //if root is a leaf, search item
    //else root is not a leaf, search children tree
    //
    fn search_root(&mut self, rect:MBRect) -> Option<Vec<usize>>{
        //判断root是否是leaf
        let root_id  = self.root_id;
        let mut block = self.bfa.get(root);
        let root_node = Node::from_block(& mut block);
        if !self.is_leaf(&root_node){
            //如果root不是leaf
            //判断root的rect是否与要寻的rect有交集
            let rect_root = root_node.rect();
            if rect.overlap(rect_root){
                //若有交集
                let mut erg: Vec<usize> = Vec::new();
                //向下遍历子树
                for i in 0.. root_node.content.len()-1{
                    //若root子树不是leaf 继续向下遍历
                    let mut tmp_block = self.bfa.get(i);
                    let tmp_node = Node::from_block(&mut tmp_block);
                    let tmp_id = tmp_node.id;
                    let mut tmp_gruppe: Vec<usize> = Vec::new();
                    if !self.is_leaf(&tmp_node){
                        tmp_gruppe = self.search_overlap_gruppe(&rect,tmp_id);
                    }
                    //第一层root 判定是否为leaf
                    //第二层从root出发 判定root下的子树 得到交集vector
                    //递归判定之前得到的vector里的第一个值是否为leaf
                    //因为leaf都在同一层面
                    //da alle leaf in gleicher Etage
                    while !(Node::from_block(& mut (self.bfa.get(tmp_gruppe[0])))).leaf{
                        let neu_gruppe = self.search_overlap_gruppes(&rect, tmp_gruppe);
                        tmp_gruppe = neu_gruppe;
                    }
                    //直到子树是leaf 直接查找leaf的item
                    //将所有符合leaf的item里的加入erg中一起返回
                    for i in 0..tmp_gruppe.len()-1{
                        let mut erg_block = self.bfa.get(i);
                        let erg_node = Node::from_block(&mut erg_block);
                        let erg_id = erg_node.id;
                        let erg_part = self.search_leaf(&rect, erg_id);
                    }
                    for i in 0..erg_part.len()-1{
                        erg.push(erg_part[i]);
                    }
                    return Some(erg);
                }
            }
        } else{
            //如果root是leaf
            let rect_leaf = root_node.rect();
            //判断leaf的rect是否与要寻的rect有交集
            return if rect.overlap(rect_leaf){
                Some(block.contents as Vec<usize>)
            }
            else { return None; }
        }
        None
    }


    //get overlapped children from a node
    //tmp is not leaf
    fn search_overlap_gruppe(&mut self, rect: &MBRect, tmp:usize) -> Vec<usize>{
        let mut tmp_block = self.bfa.get(tmp);
        let tmp_node = Node::from_block(& mut tmp_block);
        if !self.is_leaf(&tmp_node){
            let children = tmp_block.contents;
            let mut overlapped:Vec<usize> = Vec::new();
            for i in 0..children.len()-1{
                let mut child = children[i] as usize  ;
                let mut child_block = self.bfa.get(child);
                let mut child_node = Node::from_block(& mut child_block);
                let mut child_rect = child_node.rect();
                if rect.overlap(child_rect) {
                    let child_id = child_node.id;
                    overlapped.push(child_id);
                }
            }
        }
       overlapped
    }

    //get overlapped children from nodes
    //tmp is not leaf
    fn search_overlap_gruppes(&mut self, rect: &MBRect, tmp: Vec<usize>) -> Vec<usize>{
        let mut overlapped:Vec<usize> = Vec::new();
        //遍历所有tmp
        for i in 0..tmp.len()-1{
            let mut tmp_block = self.bfa.get(i);
            let tmp_node = Node::from_block(& mut tmp_block);
            if !self.is_leaf(&tmp_node){
                let children = tmp_block.contents;
                for i in 0..children.len()-1{
                    let mut child = children[i] as usize  ;
                    let mut child_block = self.bfa.get(child);
                    let mut child_node = Node::from_block(& mut child_block);
                    let mut child_rect = child_node.rect();
                    if rect.overlap(child_rect) {
                        let child_id= child_node.id;
                        overlapped.push(child_id);
                    }
                }
            }
        }
        overlapped
    }


    //root is a leaf node
    fn search_leaf(&mut self, rect:&MBRect, & tmp:usize) -> Vec<usize> {
        let mut tmp_block = self.bfa.get(tmp);
        let tmp_node = Node::from_block(& mut tmp_block);
        let mut erg: Vec<usize> = Vec::new();
        if self.is_leaf(&tmp_node){
            let rect_leaf = tmp_node.rect();
            if rect.overlap(rect_leaf){
                erg = tmp.block.contents as Vec<usize>;
            }
        }
        erg
    }


////////////////////////////////////////////////////////////////////////////////
    //Hilfsfunktion fuer choose_leaf und choose_leaves
    fn add_area(&mut self, small_rect: &MBRect, big_rect: &MBRect) -> f64{
        let sbl = &small_rect.botton_left;
        let str = &small_rect.top_right;
        let bbl = &big_rect.botton_left;
        let btr = &big_rect.top_right;
        let area =  (btr.y - bbl.y) * (btr.x - bbl.x);
        let mut add_area: f64 = 0 as f64;
        if sbl >= bbl && str <= btr{ }
        else {
            let neu_bl_x = min(sbl.x,bbl.x);
            let neu_bl_y = min(sbl.y,bbl.y);
            let neu_tr_x = max(str.x,btr.x);
            let neu_tr_y = max(str.y,btr.y);
            let neu_area = (neu_tr_y - neu_bl_y) * (neu_tr_x - neu_bl_x);
            add_area = neu_area - area;
        }
        add_area
    }

    //Hilfsfunktion fuer insert
    fn choose_leaf(&mut self, insert_rect:&MBRect) -> Vec<usize> {
        let root_id  = self.root_id;
        let mut block = self.bfa.get(root);
        let root_node = Node::from_block(& mut block);
        let botton_left = &insert_rect.botton_left;
        let top_right = &insert_rect.top_right;
        //es kann sein, dass ein Rect mit zwei Rect mit gleichem add_area ueberlappt
        let mut child_id: Vec<usize> = Vec::new();
        //如果root是leaf 则返还root
        if self.is_leaf(&root_node) {
            child_id.push(root_id);
        } else {//root不是leaf
            //选择root的子节点，使得将新的点加入该子节点中，子节点增加的面积最少
            let mut area: f64 = (top_right.y - botton_left.y) *
                (top_right.x - botton_left.x) as f64;
            for i in 0..root_node.content.len()-1 {
                let mut tmp_block = self.bfa.get(i);
                let tmp_node = Node::from_block(& mut tmp_block);
                let tmp_rect = tmp_node.rect();
                let tmp_id = tmp_node.id;
                let tmp_area = self.add_area(insert_rect, &tmp_rect);
                if tmp_area <= area {
                    area = tmp_area;
                    child_id.push(tmp_id);
                }
            }
            //但是！可能同时有多个子节点符合这个筛选条件
            //那么当child id的第一个不为leaf时 向下迭代遍历子节点
            while !self.is_leaf(&Node::from_block(&mut self.bfa.get(child_id[0]))){
                let mut child_child_id:Vec<usize> = Vec::new();
                for i in 0..child_id.len()-1{
                    child_child_id.push(child_id[0]);
                }
                child_id = self.choose_leaves(insert_rect,child_child_id);
            }

        }
        child_id
    }

    //Hilfsfunktion fuer insert
    fn choose_leaves(&mut self, insert_rect: &MBRect, nodes_id: Vec<usize>) -> Vec<usize> {
        let mut child_id: Vec<usize> = Vec::new();
        for i in nodes_id.len()-1{
            let mut tmp_block = self.bfa.get(nodes_id[i]);
            let tmp_node = Node::from_block(& mut tmp_block);
            let botton_left = &insert_rect.botton_left;
            let top_right = &insert_rect.top_right;


            let mut area: f64 = (top_right.y - botton_left.y) *
                (top_right.x - botton_left.x) as f64;
            for i in 0..tmp_node.content.len()-1 {
                let mut tmp_block2 = self.bfa.get(i);
                let tmp_node2 = Node::from_block(& mut tmp_block2);
                let tmp_rect = tmp_node2.rect();
                let tmp_id = tmp_node.id;
                let tmp_area = self.add_area(insert_rect, &tmp_rect);
                if tmp_area <= area {
                    area = tmp_area;
                    child_id.push(tmp_id);
                }
            }

        }
        child_id
    }

    //acsend from a leaf node with id to the root
    // adjusting covering rectangles
    fn adjust_tree(& mut self, id: usize){
        let mut block = self.bfa.get(id);
        let node = Node::from_block(& mut block);
        //check if done， stop
        if id == self.root_id{

        } else {//adjust covering rectangle in parent entry
            //wie bekommt man parent node?
            //TODO/////

        }

    }

    //add a new entry to a full node with M children
    //divide the collection of M+1 entries between two nodes
    //linear cost algorithm
    //在M+1个entries中选择2个作为两个新组的第一个elem
    //如何选择这两个entries？
    //选择浪费最大面积的一组 如果他们被放在了相同的组那最棒了
    //覆盖两个entries的矩形面积减去他们各自的矩形的面积
    //剩余entries将按次分配到一组
    //每次都会计算将剩余entries添加到组所需的面积扩展
    //分配的entry展示了两组间最大的差
    
    fn split(& mut self, id: usize){
        let mut assigned_1: Vec<&usize> = Vec::new();
        let mut assigned_2: Vec<&usize> = Vec::new();
        //S1: pick first entry for each group, PickSeeds
        let first_one = self.pick_seeds().get(0).unwrap();
        assigned_1.push(first_one);
        let first_two = self.pick_seeds().get(1).unwrap();
        assigned_2.push(first_two);
        //S2: check if done

        //S3: select entry to assign, PickNext, repeat S2


    }

    //12 13 14 15 16 23 24 25 26
    //Hilfsfkt fuer split
    //select two entries to be the first elements of the groups
    fn pick_seeds(&mut self) -> Vec<usize>{
        let mut res:Vec<usize> = Vec::new();
        let mut e1 =0 as usize;
        let mut e2 = 0 as usize;
        let mut largest_d = 0 as f64;
        let root = self.root_id;
        for i in 1..self.total_id{
            let tmp_node_1 = Node::from_block(&mut self.bfa.get(i));
            let tmp_rect_1 = tmp_node_1.rect();
            let area_e1=(tmp_rect_1.top_right.y-tmp_rect_1.botton_left.y) *
                (tmp_rect_1.top_right.x-tmp_rect_1.botton_left.x);
            for j in i+1..self.total_id{
                let tmp_node_2 = Node::from_block(&mut self.bfa.get(j));
                let tmp_rect_2 = tmp_node_2.rect();
                let area_e2=(tmp_rect_2.top_right.y-tmp_rect_2.botton_left.y) *
                    (tmp_rect_2.top_right.x-tmp_rect_2.botton_left.x);
                let rect_mbr = MBRect::mbr_of_rects(&tmp_rect_1, &tmp_rect_2,self.total_id+1);
                let area_mbr = (rect_mbr.top_right.y - rect_mbr.botton_left.y)*
                    (rect_mbr.top_right.x - rect_mbr.botton_left.x);
                let area_waste = area_mbr - area_e1 - area_e2;
                if area_waste > largest_d {
                    largest_d = area_waste;
                    e1 = i;
                    e2 = j;
                }
            }
        }
        res.push(e1);
        res.push(e2);
        res
    }

    //Hilfsfkt fuer split
    //select one remaining entry for classification in a group
    fn pick_next(&mut self, assigned: Vec<usize>, rect1: &MBRect, rect2: &MBRect) -> usize{
        let mut d1: f64 = 0 as f64;
        let mut d2: f64 = 0 as f64;
        let mut max_d: f64 = 0 as f64;
        let mut pref: usize = 0 as usize;

        for i in 0..self.total_id-1{
            let mut tmp_block = self.bfa.get(i);
            let tmp_node = Node::from_block(& mut tmp_block);
            let tmp_rect = tmp_node.rect();
            //for each entry not yet in a group

            if !assigned.contains(&i) {
                //calculate the area increase required in the covering rectangle of group1
                //and of group 2
                d1 = self.add_area(&tmp_rect, rect1);
                d2 = self.add_area(&tmp_rect, rect2);
                if d2 > d1 && d2 - d1 > max_d {max_d = d2 - d1; pref = i;}
                else if d1 <= d2 && d1 - d2 > max_d {max_d = d1 - d2; pref = i;}
            }
        }

        pref
    }


}






use serde::{Serialize, Deserialize};
use bincode::{serialize, deserialize};
use core::borrow::Borrow;
use std::cmp::{min, max};
use std::intrinsics::breakpoint;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Student{
    first_name: String,
    last_name: String,
    matr_nr: u32,
}
impl Student {
    pub fn new(fname: &str, lname: &str, matrnr: u32) -> Self {
        Student {
            first_name: fname.to_string(),
            last_name: lname.to_string(),
            matr_nr: matrnr,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn vec_to_block(vec:Vec<u8>) -> Block{
        Block::new(vec)
    }

    pub fn deserialize(input: &Vec<u8>) -> Student {
        bincode::deserialize(input).unwrap()
    }

    /*extern crate serde;
    extern crate serde_json;
    use serde_derive::*;
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Student {
        vorname: String,
        nachname: String,
        matrnr: usize
    }

    impl Student{
        pub fn new(vor:&str,nach:&str,nr:&str) -> Student{
            let vorname = String::from(vor);
            let nachname = String::from(nach);
            let matrnr = String::from(nr).parse::<usize>().unwrap();
            Student{vorname,nachname,matrnr}
        }

        pub fn serialize(&mut self) -> Option<Block>{
            let student_json = serde_json::to_vec(self);
            match student_json {
                Ok(student) => {
                    Some(Block::new(student))
                }
                Err(error) => None
            }
        }

        pub fn serialize1(&mut self) -> Block{
            let mut vec = Vec::new();

            self.vorname.as_bytes();
            self.nachname.as_bytes().to_vec();
            vec.push(self.matrnr as u8);

            let block = Block::new(vec);
            block
        }

        pub fn deserialize(block: Block) -> Option<Student>{
            let bytes = block.contents;
            let s = String::from_utf8(bytes).expect("Found invalid UTF-8");
            //println!("{}", s);
            let student = serde_json::from_str(&s);
            match student{
                Ok(student) => {
                    Some(student)
                }
                Err(error) => None
            }
        }*/
}


impl Block{
    pub fn new(contents:Vec<u8>) -> Block{
        Block{contents}
    }
}


impl BFA {
    pub fn new(block_size:usize, path:&str) -> BFA{

        let filepath = format!("{}",path);
        let updatepath = format!("{}updated",path);
        let metadatenpath = format!("{}metadaten",path);

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path);

        match file {
            //zwei faelle
            //1. File update_file existiert schon
            //bildet vector update_file aus dem File update_file

            Ok(mut file) => {
                Some(&file);

                let mut update_file: Vec<bool> = vec![true; block_size];
                let update = File::open(& updatepath);
                match update {
                    Ok(mut updated) => {
                        let mut vec: Vec<u8> = vec![0; file.metadata().unwrap().len() as usize];
                        updated.seek(SeekFrom::Start(0));
                        updated.read(&mut vec);

                        for i in 0..vec.len() {
                            if vec[i] == 0 {
                                update_file[i] = false;
                            }
                        }
                    }
                    Err(e) => {
                        println!("not exists")
                    }
                }

                let reserved_file: Vec<bool> = Vec::new();
                let mut metadaten = HashMap::new();
                metadaten.insert("path".to_string(),path.to_string());
                metadaten.insert("updated".to_string(),updatepath.to_string());
                metadaten.insert("metadaten".to_string(),metadatenpath.to_string());

                let reserve_count = update_file.len();

                BFA { block_size, file, metadaten, update_file, reserved_file, reserve_count }
            }
            //1. File update_file extiert schon
            //bildet vector update_file ganz neu
            Err(error) => {
                let new = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path).expect("error");

                let update_file: Vec<bool> = vec![true; block_size];
                let reserved_file: Vec<bool> = Vec::new();
                let mut metadaten = HashMap::new();
                metadaten.insert("path".to_string(),path.to_string());
                metadaten.insert("updated".to_string(),updatepath.to_string());
                metadaten.insert("metadaten".to_string(),metadatenpath.to_string());
                let reserve_count = 0 as usize;

                BFA { block_size, file: new, metadaten, update_file, reserved_file, reserve_count }
            }
        }


        /*let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path).expect("error");

        let update_file: Vec<bool> = vec![true;block_size];
        let reserved_file: Vec<bool> = Vec::new();
        let mut metadaten = HashMap::new();
        let reserve_count = 0 as usize;

        BFA{block_size,file,metadaten,update_file,reserved_file,reserve_count}*/
    }

    pub fn get(&mut self, &id:usize) -> Block{
        let mut vec:Vec<u8> = vec![0;self.block_size];

        if self.update_file[id] {
            let start = (id * self.block_size) as u64;

            self.file.seek(SeekFrom::Start(start));
            self.file.read(&mut vec);
            self.file.seek(SeekFrom::Start(0));
        }
        else { println!("update data not found") }

        let block = Block::new(vec);
        block
    }

    pub fn update(&mut self,id:usize, block: Block){
        //nach update, ist reserved_file[id] falsch, wird nie wieder zu true gesetzt
        //wenn man die noch benutzen will, guckt man dann im update_file nach
        if self.reserved_file[id] || self.update_file[id]{
            let start = (&id * self.block_size) as u64;
            self.file.seek(SeekFrom::Start(start));
            self.file.write(&block.contents);
            self.file.seek(SeekFrom::Start(0));
            self.reserved_file[&id] = false;
            self.update_file[&id] =true;
        }
    }

    pub fn insert(&mut self, block:Block) -> u64{
        let id = self.reserve();
        self.update(id,block);
        id as u64
    }

    pub fn contains(&mut self, id:usize) -> bool{
        let mut bool = false;
        if id > self.update_file.len() {
            println!("id: {} too large", id);
        }
        else{
            bool = self.update_file[id]
        }
        bool
    }

    pub fn remove(&mut self, id:usize) {
        if id > self.update_file.len() {
            println!("id: {} too large", id);
        }
        else {
            self.update_file[id] = false;
        }
    }

    pub fn reserve(&mut self) -> usize{
        let count = self.reserve_count;
        self.reserved_file.insert(count,true);
        self.reserve_count += 1;
        count
    }



    pub fn close(&mut self){
        self.reserved_file =  Vec::new();
        let mut updated_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("updated file")
            .expect("error");

        for i in 0 .. self.update_file.len() {
            if self.update_file[i]{
                write!(updated_file,"1");
            } else {
                write!(updated_file,"0");
            }
        }
        //self.update_file = vec![true;self.block_size];

    }

    pub fn get_metadaten(&mut self) {
        /*let mut daten = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("metadaten.txt").expect("error");

        let metadata = self.file.metadata().expect("error");
        self.metadaten.insert(String::from("Length"), format!(":{}", metadata.len()));
        self.metadaten.insert("Is dir".to_string(),format!(":{}",metadata.is_dir()));

        for key in self.metadaten.keys(){
            let mut value = self.metadaten.get(key).expect("error");
           // print!("{}", value);
            daten.write(key.as_bytes());
            daten.write(value.as_bytes());
            daten.write_all(b"\n");
        }*/

    }

    pub fn get_root(& mut self) -> usize{
        let mut root = self.metadaten.get("path").expect("no root");
        let root = root.parse::<usize>().expect("invalid root");
        return root;
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;
    use std::fmt::Error;
    use super::*;



    #[test]
    fn test_bfa_get_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        file.write_all(b"HelloWorld").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        let mut block = bfa_1.get(0);
        assert_eq!(block.contents, [72, 101, 108, 108, 111]);
        Ok(())
    }

    #[test]
    fn test_reserve_ok() -> Result<(),Error>{
        let block_size = 5 as usize;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        file.write_all(b"HelloWorld").expect("error");
        bfa_1.reserve();
        bfa_1.reserve();
        assert_eq!(bfa_1.reserved_file,[true,true]);
        Ok(())
    }

    #[test]
    fn test_update_ok() -> Result<(),Error>{
        let block_size = 5;
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(block_size,"Hello.txt");
        bfa_1.file.write_all(b"HelloWorld").expect("error");
        let block_1 =bfa_1.get(0);
        bfa_1.reserve();
        bfa_1.reserve();
        bfa_1.update(1,block_1);

        let mut b = String::new();
        bfa_1.file.read_to_string(& mut b);
        assert_eq!(b, "HelloHello".to_string());
        Ok(())
    }

    #[test]
    fn test_get_metadaten_ok() -> Result<(),Error>{
        let mut file = File::create("Hello.txt").expect("error");
        let mut bfa_1 = BFA::new(5, "Hello.txt");
        file.write_all(b"HelloWorld").expect("error");
        bfa_1.get_metadaten();

        Ok(())
    }

    /*#[test]
    fn test_student_block_ok() -> Result<(),Error>{
        let mut student_1 = Student::new("Ling","Feng",2719983);
        let mut student_2 = Student::new("Yanping","Long",2767970);

        let student_1_block = student_1.serialize().unwrap();
        let bytes_1 = student_1_block.contents;
        let s_1 = String::from_utf8(bytes_1).expect("Found invalid UTF-8");
        assert_eq!(s_1,"{\"vorname\":\"Ling\",\"nachname\":\"Feng\",\"matrnr\":2719983}");

        let student_2_block = student_2.serialize().unwrap();
        let bytes_2 = student_2_block.contents;
        let s_2 = String::from_utf8(bytes_2).expect("Found invalid UTF-8");
        assert_ne!(s_2,"{\"vorname\":\"Ling\",\"nachname\":\"Feng\",\"matrnr\":2719983}");

        let student_3_block = student_1.serialize().unwrap();
        let student_3 = Student::deserialize(student_3_block).unwrap();
        assert_eq!(student_1.vorname,student_3.vorname);
        assert_eq!(student_1.nachname,student_3.nachname);
        assert_eq!(student_1.matrnr,student_3.matrnr);

        Ok(())
    }*/

    #[test]
    fn test_student_ok(){
        let s1 = Student::new("ling", "feng", 2719983);
        let s2 = Student::new("yanping", "long", 2767970);
        let serialized1:Vec<u8> = s1.serialize();
        let serialized2:Vec<u8> = s2.serialize();
        let deserialized1:Student = Student::deserialize(&serialized1);
        let deserialized2:Student = Student::deserialize(&serialized2);
        assert_eq!(s1, deserialized1);
        assert_eq!(s2, deserialized2);

    }



}



