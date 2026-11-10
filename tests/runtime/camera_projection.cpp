#include <cassert>
#include <climits>
#include <cstdio>
#include <initializer_list>
#include "../../runtime/camera_projection.hpp"

using namespace epok::camera_projection;
int main() {
    assert(focal(90*4096)==4096&&focal(0)==focal(25*4096)&&focal(INT_MAX)==focal(120*4096));
    assert(focal(25*4096)>focal(45*4096)&&focal(45*4096)>4096&&focal(120*4096)<4096);
    for(const int width:{320,640}) {
        const int height=width*3/4;
        int32_t p[3]={0,0,4096},out[2]={-17,-19};
        assert(point(p,4096,4096,width,height,out));
        assert(out[0]==width/2*4096&&out[1]==height/2*4096);
        p[2]=1024;assert(point(p,4096,4096,width,height,out));
        for(const int32_t z:{INT_MIN,-1,0,1023,128*4096,INT_MAX}) {
            p[2]=z;out[0]=-17;out[1]=-19;assert(!point(p,4096,4096,width,height,out));assert(out[0]==-17&&out[1]==-19);
        }
        p[2]=4096;p[0]=-4096;assert(point(p,4096,4096,width,height,out)&&out[0]==0);
        --p[0];assert(!point(p,4096,4096,width,height,out));
        p[0]=4096;assert(!point(p,4096,4096,width,height,out));
        --p[0];assert(point(p,4096,4096,width,height,out)&&out[0]<width*4096);
        p[0]=0;p[1]=3072;assert(point(p,4096,4096,width,height,out)&&out[1]==0);
        ++p[1];assert(!point(p,4096,4096,width,height,out));
        p[1]=-3072;assert(!point(p,4096,4096,width,height,out));
        ++p[1];assert(point(p,4096,4096,width,height,out)&&out[1]<height*4096);
        p[0]=2048;p[1]=0;int32_t mirrored[2];
        assert(point(p,4096,4096,width,height,out)&&point(p,-4096,4096,width,height,mirrored));
        assert(out[0]+mirrored[0]==width*4096&&out[1]==mirrored[1]);
        for(const int degrees:{25,45,90,120}) {
            const int f=focal(degrees*4096);p[2]=1024;
            for(const int32_t extreme:{INT_MIN,INT_MAX}) {p[0]=extreme;assert(!point(p,f,f,width,height,out));p[0]=0;p[1]=extreme;assert(!point(p,f,f,width,height,out));p[1]=0;}
        }
    }
    int32_t p[3]={0,0,4096},out[2];
    assert(!point(nullptr,4096,4096,320,240,out)&&!point(p,4096,4096,320,240,nullptr));
    assert(!point(p,4096,4096,0,240,out)&&!point(p,INT_MIN,4096,320,240,out));
    std::puts("Camera projection: shared Q12 lens, viewport/near/far boundaries, mirror and overflow rejection passed");
}
